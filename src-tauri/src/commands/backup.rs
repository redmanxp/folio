use crate::error::{AppError, AppResult};
use crate::state::AppState;
use rusqlite::Connection;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

const REQUIRED_TABLES: &[&str] = &[
    "schema_migrations",
    "entities",
    "providers",
    "documents",
    "payments",
    "settings",
];

/// Límites anti zip-bomb en restauración.
const MAX_ZIP_ENTRIES: usize = 10_000;
const MAX_ZIP_UNCOMPRESSED_BYTES: u64 = 500 * 1024 * 1024; // 500 MiB
const MAX_ZIP_ENTRY_BYTES: u64 = 100 * 1024 * 1024; // 100 MiB
const MAX_ZIP_COMPRESSION_RATIO: u64 = 100;

#[tauri::command]
pub fn create_backup(state: State<'_, AppState>) -> AppResult<String> {
    create_backup_inner(&state)
}

#[cfg(test)]
pub(crate) fn create_backup_for_test(state: &AppState) -> AppResult<String> {
    create_backup_inner(state)
}

fn create_backup_inner(state: &AppState) -> AppResult<String> {
    let paths = state.paths_snapshot()?;
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = paths
        .backups_dir
        .join(format!("folio_backup_{stamp}.zip"));

    state.with_db(|conn| {
        conn.execute_batch("PRAGMA wal_checkpoint(FULL);")?;
        Ok(())
    })?;

    let file = File::create(&backup_path)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    add_file_to_zip(&mut zip, &paths.db_path, Path::new("folio.db"), options)?;

    let docs_root = &paths.documents_dir;
    if docs_root.exists() {
        for entry in WalkDir::new(docs_root).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(docs_root)
                    .map_err(|_| AppError::msg("Ruta de documento inválida"))?;
                let zip_path = Path::new("documents").join(rel);
                add_file_to_zip(&mut zip, entry.path(), &zip_path, options)?;
            }
        }
    }

    zip.finish()?;
    log::info!(
        "Backup creado: {}",
        backup_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("backup.zip")
    );
    Ok(backup_path.to_string_lossy().into_owned())
}

fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    source: &Path,
    zip_path: &Path,
    options: SimpleFileOptions,
) -> AppResult<()> {
    let name = zip_path
        .to_str()
        .ok_or_else(|| AppError::msg("Nombre de archivo inválido en backup"))?
        .replace('\\', "/");
    zip.start_file(name, options)?;
    let mut f = File::open(source)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    zip.write_all(&buffer)?;
    Ok(())
}

#[tauri::command]
pub fn restore_backup(backup_path: String, state: State<'_, AppState>) -> AppResult<()> {
    restore_backup_inner(&state, &backup_path)
}

#[cfg(test)]
pub(crate) fn restore_backup_for_test(state: &AppState, backup_path: &str) -> AppResult<()> {
    restore_backup_inner(state, backup_path)
}

/// Restauración segura:
/// 1) extrae a staging sin tocar datos vivos
/// 2) valida ZIP + SQLite
/// 3) guarda snapshot de seguridad del estado actual
/// 4) reemplaza solo si todo está listo
/// 5) reabre DB; si falla, rollback
fn restore_backup_inner(state: &AppState, backup_path: &str) -> AppResult<()> {
    let path = PathBuf::from(backup_path);
    if !path.is_file() {
        return Err(AppError::msg("El archivo de backup no existe"));
    }

    let paths = state.paths_snapshot()?;
    let work = paths.data_dir.join("_restore_work");
    // Snapshot fuera de `_restore_work` para sobrevivir limpieza tras crash a mitad.
    let safety = paths.data_dir.join("_restore_safety");
    let incoming = work.join("incoming");

    // Limpiar restos de intentos previos (no toca db/docs vivos).
    if work.exists() {
        std::fs::remove_dir_all(&work)?;
    }
    if safety.exists() {
        std::fs::remove_dir_all(&safety)?;
    }
    std::fs::create_dir_all(&incoming)?;

    // 1-2) Preparar + validar sin tocar datos vivos.
    let restored_db = incoming.join("folio.db");
    if let Err(err) = (|| -> AppResult<()> {
        extract_zip_safe(&path, &incoming)?;
        validate_restored_backup(&incoming, &restored_db)?;
        Ok(())
    })() {
        let _ = std::fs::remove_dir_all(&work);
        return Err(err);
    }

    // 3) Snapshot de seguridad del estado actual (mientras la DB sigue abierta).
    if let Err(err) = (|| -> AppResult<()> {
        state.with_db(|conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(FULL);")?;
            Ok(())
        })?;
        std::fs::create_dir_all(&safety)?;
        std::fs::copy(&paths.db_path, safety.join("folio.db"))?;
        for suffix in ["-wal", "-shm"] {
            let side = PathBuf::from(format!("{}{suffix}", paths.db_path.display()));
            if side.exists() {
                let _ = std::fs::copy(&side, safety.join(format!("folio.db{suffix}")));
            }
        }
        let safety_docs = safety.join("documents");
        if paths.documents_dir.exists() {
            copy_dir_recursive(&paths.documents_dir, &safety_docs)?;
        } else {
            std::fs::create_dir_all(&safety_docs)?;
        }
        Ok(())
    })() {
        let _ = std::fs::remove_dir_all(&work);
        let _ = std::fs::remove_dir_all(&safety);
        return Err(AppError::msg(format!(
            "No se pudo preparar el snapshot de seguridad; datos actuales intactos. Detalle: {err}"
        )));
    }

    // Desconectar DB viva recién ahora.
    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo bloquear la base de datos"))?;
        *guard = Connection::open_in_memory()?;
    }

    // 4) Reemplazar sobre directorios temporales y luego swap.
    let replace_result = (|| -> AppResult<()> {
        let new_docs = work.join("new_documents");
        if incoming.join("documents").exists() {
            copy_dir_recursive(&incoming.join("documents"), &new_docs)?;
        } else {
            std::fs::create_dir_all(&new_docs)?;
        }

        let new_db = work.join("new_folio.db");
        std::fs::copy(&restored_db, &new_db)?;

        // Validar una vez más la copia que vamos a activar.
        validate_sqlite_file(&new_db)?;

        let old_docs = work.join("old_documents");
        if paths.documents_dir.exists() {
            if std::fs::rename(&paths.documents_dir, &old_docs).is_err() {
                copy_dir_recursive(&paths.documents_dir, &old_docs)?;
                std::fs::remove_dir_all(&paths.documents_dir)?;
            }
        }

        // Activar documentos nuevos.
        if std::fs::rename(&new_docs, &paths.documents_dir).is_err() {
            copy_dir_recursive(&new_docs, &paths.documents_dir)?;
            let _ = std::fs::remove_dir_all(&new_docs);
        }

        // Activar DB nueva (reemplazo atómico vía archivo temporal + rename).
        let db_tmp = paths.db_path.with_extension("db.incoming");
        std::fs::copy(&new_db, &db_tmp)?;
        // Quitar sidecars WAL viejos antes de activar.
        for suffix in ["-wal", "-shm"] {
            let side = PathBuf::from(format!("{}{suffix}", paths.db_path.display()));
            let _ = std::fs::remove_file(side);
        }
        if std::fs::rename(&db_tmp, &paths.db_path).is_err() {
            std::fs::copy(&db_tmp, &paths.db_path)?;
            let _ = std::fs::remove_file(&db_tmp);
        }

        Ok(())
    })();

    if let Err(err) = replace_result {
        log::error!("Fallo al reemplazar datos durante restore: {err}");
        let _ = rollback_from_safety(state, &paths.db_path, &paths.documents_dir, &safety);
        let _ = std::fs::remove_dir_all(&work);
        let _ = std::fs::remove_dir_all(&safety);
        return Err(AppError::msg(format!(
            "No se pudo completar la restauración; se intentó rollback. Detalle: {err}"
        )));
    }

    // 5) Confirmar: reabrir SQLite.
    match crate::db::open_db(&paths.db_path) {
        Ok(conn) => {
            {
                let mut guard = state
                    .db
                    .lock()
                    .map_err(|_| AppError::msg("No se pudo reabrir la base de datos"))?;
                *guard = conn;
            }
            let _ = std::fs::remove_dir_all(&work);
            let _ = std::fs::remove_dir_all(&safety);
            log::info!(
                "Backup restaurado: {}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("backup.zip")
            );
            Ok(())
        }
        Err(err) => {
            log::error!("DB restaurada no abre; rollback: {err}");
            rollback_from_safety(state, &paths.db_path, &paths.documents_dir, &safety)?;
            let _ = std::fs::remove_dir_all(&work);
            let _ = std::fs::remove_dir_all(&safety);
            Err(AppError::msg(format!(
                "La base restaurada no es usable; se revirtió al estado anterior. Detalle: {err}"
            )))
        }
    }
}

fn rollback_from_safety(
    state: &AppState,
    db_path: &Path,
    documents_dir: &Path,
    safety: &Path,
) -> AppResult<()> {
    let safety_db = safety.join("folio.db");
    let safety_docs = safety.join("documents");
    if !safety_db.exists() {
        return Err(AppError::msg(
            "Falló la restauración y no hay snapshot de seguridad para rollback",
        ));
    }

    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo bloquear la base durante rollback"))?;
        *guard = Connection::open_in_memory()?;
    }

    if documents_dir.exists() {
        let _ = std::fs::remove_dir_all(documents_dir);
    }
    if safety_docs.exists() {
        copy_dir_recursive(&safety_docs, documents_dir)?;
    } else {
        std::fs::create_dir_all(documents_dir)?;
    }

    for suffix in ["-wal", "-shm"] {
        let side = PathBuf::from(format!("{}{suffix}", db_path.display()));
        let _ = std::fs::remove_file(side);
    }
    std::fs::copy(&safety_db, db_path)?;

    let conn = crate::db::open_db(db_path)?;
    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo reabrir la base tras rollback"))?;
        *guard = conn;
    }
    log::warn!("Rollback de restauración completado desde snapshot de seguridad");
    Ok(())
}

fn extract_zip_safe(zip_path: &Path, dest: &Path) -> AppResult<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::msg(format!("ZIP inválido o corrupto: {e}")))?;

    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(AppError::msg(format!(
            "Backup con demasiadas entradas (máx. {MAX_ZIP_ENTRIES})"
        )));
    }

    let mut total_uncompressed: u64 = 0;
    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| AppError::msg(format!("No se pudo leer entrada del ZIP: {e}")))?;
        let raw_name = entry.name().replace('\\', "/");
        let rel = sanitize_zip_relative_path(&raw_name)?;
        let out_path = dest.join(&rel);

        if entry.is_dir() || raw_name.ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }

        let declared = entry.size();
        if declared > MAX_ZIP_ENTRY_BYTES {
            return Err(AppError::msg(format!(
                "Entrada ZIP demasiado grande (máx. {} MiB): {}",
                MAX_ZIP_ENTRY_BYTES / (1024 * 1024),
                rel.display()
            )));
        }
        let compressed = entry.compressed_size().max(1);
        if declared / compressed > MAX_ZIP_COMPRESSION_RATIO {
            return Err(AppError::msg(format!(
                "Entrada ZIP con ratio de compresión sospechoso: {}",
                rel.display()
            )));
        }
        total_uncompressed = total_uncompressed.saturating_add(declared);
        if total_uncompressed > MAX_ZIP_UNCOMPRESSED_BYTES {
            return Err(AppError::msg(format!(
                "Backup descomprimido supera el límite (máx. {} MiB)",
                MAX_ZIP_UNCOMPRESSED_BYTES / (1024 * 1024)
            )));
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = File::create(&out_path)?;
        // Copiar con tope real por si el header miente.
        let mut limited = entry.take(MAX_ZIP_ENTRY_BYTES + 1);
        let written = std::io::copy(&mut limited, &mut outfile)?;
        if written > MAX_ZIP_ENTRY_BYTES {
            let _ = std::fs::remove_file(&out_path);
            return Err(AppError::msg(format!(
                "Entrada ZIP excedió el tamaño máximo al extraer: {}",
                rel.display()
            )));
        }
    }
    Ok(())
}

fn sanitize_zip_relative_path(name: &str) -> AppResult<PathBuf> {
    let name = name.trim_start_matches('/');
    if name.is_empty() {
        return Err(AppError::msg("Entrada ZIP con nombre vacío"));
    }
    let path = Path::new(name);
    if path.is_absolute() {
        return Err(AppError::msg(format!(
            "Entrada ZIP con ruta absoluta no permitida: {name}"
        )));
    }
    let mut clean = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(AppError::msg(format!(
                    "Entrada ZIP con path traversal no permitida: {name}"
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::msg(format!(
                    "Entrada ZIP con prefijo inválido: {name}"
                )));
            }
        }
    }
    if clean.as_os_str().is_empty() {
        return Err(AppError::msg("Entrada ZIP inválida"));
    }
    Ok(clean)
}

fn validate_restored_backup(incoming: &Path, restored_db: &Path) -> AppResult<()> {
    if !restored_db.is_file() {
        return Err(AppError::msg("El backup no contiene folio.db"));
    }
    // Solo permitimos folio.db + documents/** en la raíz del backup.
    for entry in std::fs::read_dir(incoming)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "folio.db" || name == "documents" {
            continue;
        }
        // Ignorar basura común de macOS/Windows en ZIPs.
        if name == "__MACOSX" || name == ".DS_Store" || name.starts_with('.') {
            continue;
        }
        return Err(AppError::msg(format!(
            "El backup contiene una entrada inesperada: {name}"
        )));
    }

    if incoming.join("documents").exists() {
        // Verificar que no haya symlinks raros / paths raros bajo documents.
        for entry in WalkDir::new(incoming.join("documents"))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let meta = entry.metadata().map_err(|e| {
                AppError::msg(format!("No se pudo leer metadatos del backup: {e}"))
            })?;
            if meta.file_type().is_symlink() {
                return Err(AppError::msg(format!(
                    "El backup contiene un enlace simbólico no permitido: {}",
                    entry.path().display()
                )));
            }
        }
    }

    validate_sqlite_file(restored_db)
}

fn validate_sqlite_file(db_path: &Path) -> AppResult<()> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::msg(format!("No se pudo abrir folio.db del backup: {e}")))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| AppError::msg(format!("No se pudo verificar integridad SQLite: {e}")))?;
    if integrity.to_lowercase() != "ok" {
        return Err(AppError::msg(format!(
            "folio.db del backup no pasa integrity_check: {integrity}"
        )));
    }

    for table in REQUIRED_TABLES {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )?;
        if exists == 0 {
            return Err(AppError::msg(format!(
                "folio.db del backup no tiene la tabla requerida: {table}"
            )));
        }
    }

    // Smoke query básica.
    let _: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> AppResult<()> {
    std::fs::create_dir_all(to)?;
    for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
        let rel = entry
            .path()
            .strip_prefix(from)
            .map_err(|_| AppError::msg("Error copiando backup"))?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let paths = state.paths_snapshot()?;
    let mut items = Vec::new();
    if paths.backups_dir.exists() {
        for entry in std::fs::read_dir(&paths.backups_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("zip") {
                items.push(path.to_string_lossy().into_owned());
            }
        }
    }
    items.sort();
    items.reverse();
    Ok(items)
}

#[tauri::command]
pub fn delete_backup(backup_path: String, state: State<'_, AppState>) -> AppResult<()> {
    delete_backup_inner(&state, &backup_path)
}

#[cfg(test)]
pub(crate) fn delete_backup_for_test(state: &AppState, backup_path: &str) -> AppResult<()> {
    delete_backup_inner(state, backup_path)
}

fn delete_backup_inner(state: &AppState, backup_path: &str) -> AppResult<()> {
    let path = PathBuf::from(backup_path);
    if !path.is_file() {
        return Err(AppError::msg("El archivo de backup no existe"));
    }
    if path.extension().and_then(|e| e.to_str()) != Some("zip") {
        return Err(AppError::msg("Solo se pueden eliminar backups .zip"));
    }

    let paths = state.paths_snapshot()?;
    let backups_dir = paths
        .backups_dir
        .canonicalize()
        .unwrap_or(paths.backups_dir.clone());
    let canonical = path
        .canonicalize()
        .map_err(|_| AppError::msg("No se pudo resolver la ruta del backup"))?;
    if !canonical.starts_with(&backups_dir) {
        return Err(AppError::msg(
            "Solo se pueden eliminar backups dentro de la carpeta de backups configurada",
        ));
    }

    std::fs::remove_file(&canonical)?;
    log::info!("Backup eliminado: {:?}", canonical);
    Ok(())
}
