use crate::error::{AppError, AppResult};
use crate::state::AppState;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::State;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipArchive;
use zip::ZipWriter;

#[tauri::command]
pub fn create_backup(state: State<'_, AppState>) -> AppResult<String> {
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
    log::info!("Backup creado: {:?}", backup_path);
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
    let path = PathBuf::from(&backup_path);
    if !path.is_file() {
        return Err(AppError::msg("El archivo de backup no existe"));
    }

    let paths = state.paths_snapshot()?;

    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo bloquear la base de datos"))?;
        *guard = rusqlite::Connection::open_in_memory()?;
    }

    let staging = paths.data_dir.join("_restore_staging");
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    let file = File::open(&path)?;
    let mut archive = ZipArchive::new(file)?;
    archive.extract(&staging)?;

    let restored_db = staging.join("folio.db");
    if !restored_db.exists() {
        return Err(AppError::msg("El backup no contiene folio.db"));
    }

    let target_docs = &paths.documents_dir;
    if target_docs.exists() {
        std::fs::remove_dir_all(target_docs)?;
    }
    let staged_docs = staging.join("documents");
    if staged_docs.exists() {
        copy_dir_recursive(&staged_docs, target_docs)?;
    } else {
        std::fs::create_dir_all(target_docs)?;
    }

    std::fs::copy(&restored_db, &paths.db_path)?;
    let _ = std::fs::remove_dir_all(&staging);

    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| AppError::msg("No se pudo reabrir la base de datos"))?;
        *guard = crate::db::open_db(&paths.db_path)?;
    }

    log::info!("Backup restaurado desde {}", backup_path);
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
