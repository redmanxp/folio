use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::Document;
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

pub fn map_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<Document> {
    Ok(Document {
        id: row.get(0)?,
        entity_id: row.get(1)?,
        provider_id: row.get(2)?,
        file_name: row.get(3)?,
        original_path: row.get(4)?,
        stored_path: row.get(5)?,
        file_hash: row.get(6)?,
        mime_type: row.get(7)?,
        doc_type: row.get(8)?,
        number: row.get(9)?,
        period: row.get(10)?,
        issue_date: row.get(11)?,
        due_date: row.get(12)?,
        amount: row.get(13)?,
        currency: row.get(14)?,
        point_of_sale: row.get(15)?,
        fx_rate: row.get(16)?,
        auth_type: row.get(17)?,
        auth_code: row.get(18)?,
        receptor_doc_type: row.get(19)?,
        receptor_doc_number: row.get(20)?,
        notes: row.get(21)?,
        imported_at: row.get(22)?,
        provider_name: row.get(23)?,
        payment_status: row.get(24)?,
    })
}

pub const DOC_SELECT: &str = "
    SELECT d.id, d.entity_id, d.provider_id, d.file_name, d.original_path, d.stored_path,
           d.file_hash, d.mime_type, d.doc_type, d.number, d.period, d.issue_date, d.due_date,
           d.amount, d.currency,
           d.point_of_sale, d.fx_rate, d.auth_type, d.auth_code,
           d.receptor_doc_type, d.receptor_doc_number,
           d.notes, d.imported_at,
           p.name as provider_name, pay.status as payment_status
    FROM documents d
    LEFT JOIN providers p ON p.id = d.provider_id AND p.entity_id = d.entity_id
    LEFT JOIN payments pay ON pay.document_id = d.id AND pay.entity_id = d.entity_id
";

#[tauri::command]
pub fn get_document(
    entity_id: String,
    id: String,
    state: State<'_, AppState>,
) -> AppResult<Document> {
    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &id)?;
        let sql = format!("{DOC_SELECT} WHERE d.id = ?1 AND d.entity_id = ?2");
        conn.query_row(&sql, params![id, entity_id], map_document)
            .map_err(|_| AppError::msg("Documento no encontrado"))
    })
}

#[tauri::command]
pub fn update_document_meta(
    entity_id: String,
    id: String,
    number: Option<String>,
    period: Option<String>,
    issue_date: Option<String>,
    due_date: Option<String>,
    amount: Option<f64>,
    provider_id: Option<String>,
    notes: Option<String>,
    doc_type: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<Document> {
    state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &id)?;
        db::ensure_optional_provider_same_entity(conn, &entity_id, &provider_id)?;
        let changed = conn.execute(
            "UPDATE documents SET
                number = ?1, period = ?2, issue_date = ?3, due_date = ?4,
                amount = ?5, provider_id = ?6, notes = ?7, doc_type = ?8
             WHERE id = ?9 AND entity_id = ?10",
            params![
                number,
                period,
                issue_date,
                due_date,
                amount,
                provider_id,
                notes,
                doc_type,
                id,
                entity_id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::msg("Documento no encontrado"));
        }
        Ok(())
    })?;
    get_document(entity_id, id, state)
}

#[tauri::command]
pub fn delete_document(
    entity_id: String,
    id: String,
    delete_file: bool,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let stored_path: String = state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &id)?;
        conn.query_row(
            "SELECT stored_path FROM documents WHERE id = ?1 AND entity_id = ?2",
            params![id, entity_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::msg("Documento no encontrado"))
    })?;

    state.with_db(|conn| {
        conn.execute(
            "DELETE FROM documents WHERE id = ?1 AND entity_id = ?2",
            params![id, entity_id],
        )?;
        Ok(())
    })?;

    if delete_file {
        let path = std::path::Path::new(&stored_path);
        let docs_root = state.paths_snapshot()?.documents_dir;
        let allowed = path
            .canonicalize()
            .ok()
            .and_then(|p| {
                docs_root
                    .canonicalize()
                    .ok()
                    .map(|root| p.starts_with(root))
            })
            .unwrap_or(false);
        if allowed && path.exists() {
            std::fs::remove_file(path)?;
            log::info!("Archivo archivado eliminado");
        } else if path.exists() {
            log::warn!("No se eliminó archivo fuera de documents_dir");
        }
    }

    log::info!("Documento eliminado de la base");
    Ok(())
}

/// Abre el Explorador de Windows con el archivo seleccionado (solo bajo documents_dir).
#[tauri::command]
pub fn reveal_document_in_explorer(
    entity_id: String,
    id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let stored_path: String = state.with_db(|conn| {
        db::ensure_document_in_entity(conn, &entity_id, &id)?;
        conn.query_row(
            "SELECT stored_path FROM documents WHERE id = ?1 AND entity_id = ?2",
            params![id, entity_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::msg("Documento no encontrado"))
    })?;

    let path = std::path::PathBuf::from(&stored_path);
    if !path.is_file() {
        return Err(AppError::msg("El archivo no existe en disco"));
    }

    let docs_root = state.paths_snapshot()?.documents_dir;
    let canonical = path
        .canonicalize()
        .map_err(|e| AppError::msg(format!("No se pudo resolver la ruta del archivo: {e}")))?;
    let allowed = docs_root
        .canonicalize()
        .ok()
        .map(|root| canonical.starts_with(root))
        .unwrap_or(false);
    if !allowed {
        return Err(AppError::msg(
            "Solo se puede abrir archivos dentro de la carpeta de documentos de Folio",
        ));
    }

    #[cfg(target_os = "windows")]
    {
        // canonicalize() en Windows agrega prefijo \\?\ que explorer no entiende
        // y termina abriendo "Documentos" del usuario.
        let mut reveal = canonical.to_string_lossy().replace('/', "\\");
        if let Some(stripped) = reveal.strip_prefix(r"\\?\") {
            reveal = stripped.to_string();
        }
        // Dos args: explorer interpreta mejor /select, + ruta
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(&reveal)
            .spawn()
            .map_err(|e| AppError::msg(format!("No se pudo abrir el Explorador: {e}")))?;
        log::info!("Explorador abierto en archivo del documento");
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = canonical;
        Err(AppError::msg("Abrir en el explorador solo está disponible en Windows"))
    }
}

#[tauri::command]
pub fn list_recent_documents(
    entity_id: String,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Document>> {
    let limit = limit.unwrap_or(20).clamp(1, 100);
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let sql = format!("{DOC_SELECT} WHERE d.entity_id = ?1 ORDER BY d.imported_at DESC LIMIT ?2");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![entity_id, limit], map_document)?;
        let mut docs = Vec::new();
        for row in rows {
            docs.push(row?);
        }
        Ok(docs)
    })
}
