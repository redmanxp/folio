use crate::arca::{format_cuit, normalize_cuit_key, InvoiceData};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::extract::{extract_invoice_data_excluding, parse_invoice_text};
use crate::models::{Document, ImportReport};
use crate::state::AppState;
use crate::util::{file_sha256, guess_mime, new_id, now_iso};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tauri::State;

#[tauri::command]
pub async fn import_files(
    entity_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> AppResult<ImportReport> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || import_files_sync(state, entity_id, paths))
        .await
        .map_err(|e| AppError::msg(format!("Importación interrumpida: {e}")))?
}

fn import_files_sync(
    state: AppState,
    entity_id: String,
    paths: Vec<String>,
) -> AppResult<ImportReport> {
    if paths.is_empty() {
        return Err(AppError::msg("No se seleccionaron archivos"));
    }

    log::info!(
        "Inicio importación entidad {} ({} archivos)",
        entity_id,
        paths.len()
    );

    state.with_db(|conn| db::ensure_entity_exists(conn, &entity_id))?;

    let dest_root = state.entity_docs_dir(&entity_id)?;
    std::fs::create_dir_all(&dest_root)?;

    let mut report = ImportReport {
        imported: 0,
        duplicates: 0,
        errors: Vec::new(),
        document_ids: Vec::new(),
        qr_parsed: 0,
        ocr_used: 0,
        providers_created: 0,
    };

    for path_str in paths {
        let source = PathBuf::from(&path_str);
        match import_one(&state, &entity_id, &source, &dest_root, &mut report) {
            Ok(ImportOutcome::Imported(id)) => {
                report.imported += 1;
                report.document_ids.push(id);
            }
            Ok(ImportOutcome::Duplicate) => report.duplicates += 1,
            Err(err) => {
                log::warn!("Error importando {}: {}", path_str, err);
                report
                    .errors
                    .push(format!("{}: {}", source.display(), err));
            }
        }
    }

    log::info!(
        "Importación entidad {}: {} ok, {} dup, {} qr, {} ocr, {} proveedores nuevos, {} err",
        entity_id,
        report.imported,
        report.duplicates,
        report.qr_parsed,
        report.ocr_used,
        report.providers_created,
        report.errors.len()
    );
    Ok(report)
}

#[tauri::command]
pub async fn reprocess_documents(
    entity_id: String,
    state: State<'_, AppState>,
) -> AppResult<ImportReport> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || reprocess_documents_sync(state, entity_id))
        .await
        .map_err(|e| AppError::msg(format!("Reproceso interrumpido: {e}")))?
}

fn reprocess_documents_sync(state: AppState, entity_id: String) -> AppResult<ImportReport> {
    state.with_db(|conn| db::ensure_entity_exists(conn, &entity_id))?;

    let docs: Vec<(String, String)> = state.with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, stored_path FROM documents WHERE entity_id = ?1 ORDER BY imported_at DESC",
        )?;
        let rows = stmt.query_map(params![entity_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    })?;

    let mut report = ImportReport {
        imported: 0,
        duplicates: 0,
        errors: Vec::new(),
        document_ids: Vec::new(),
        qr_parsed: 0,
        ocr_used: 0,
        providers_created: 0,
    };

    log::info!(
        "Inicio reproceso entidad {} ({} docs)",
        entity_id,
        docs.len()
    );

    for (doc_id, stored_path) in docs {
        let path = PathBuf::from(&stored_path);
        if !path.is_file() {
            report
                .errors
                .push(format!("{stored_path}: archivo no encontrado"));
            continue;
        }
        let mut local = ImportReport {
            imported: 0,
            duplicates: 0,
            errors: Vec::new(),
            document_ids: Vec::new(),
            qr_parsed: 0,
            ocr_used: 0,
            providers_created: 0,
        };
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            enrich_existing_document(&state, &entity_id, &doc_id, &path, &mut local)
        }));
        match outcome {
            Ok(Ok(())) => {
                report.imported += 1;
                report.document_ids.push(doc_id);
                report.qr_parsed += local.qr_parsed;
                report.ocr_used += local.ocr_used;
                report.providers_created += local.providers_created;
            }
            Ok(Err(err)) => report.errors.push(format!("{stored_path}: {err}")),
            Err(_) => {
                log::error!("Panic al reprocesar {}", stored_path);
                report
                    .errors
                    .push(format!("{stored_path}: error interno al procesar"));
            }
        }
    }

    log::info!(
        "Reproceso entidad {}: {} docs, {} qr, {} ocr, {} proveedores",
        entity_id,
        report.imported,
        report.qr_parsed,
        report.ocr_used,
        report.providers_created
    );
    Ok(report)
}

#[tauri::command]
pub async fn reprocess_document(
    document_id: String,
    state: State<'_, AppState>,
) -> AppResult<ImportReport> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || reprocess_document_sync(state, document_id))
        .await
        .map_err(|e| AppError::msg(format!("Reproceso interrumpido: {e}")))?
}

fn reprocess_document_sync(state: AppState, document_id: String) -> AppResult<ImportReport> {
    let (entity_id, stored_path): (String, String) = state.with_db(|conn| {
        conn.query_row(
            "SELECT entity_id, stored_path FROM documents WHERE id = ?1",
            params![document_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| AppError::msg("Documento no encontrado"))
    })?;

    let mut report = ImportReport {
        imported: 0,
        duplicates: 0,
        errors: Vec::new(),
        document_ids: Vec::new(),
        qr_parsed: 0,
        ocr_used: 0,
        providers_created: 0,
    };

    let path = PathBuf::from(&stored_path);
    if !path.is_file() {
        report
            .errors
            .push(format!("{stored_path}: archivo no encontrado"));
        return Ok(report);
    }

    match enrich_existing_document(&state, &entity_id, &document_id, &path, &mut report) {
        Ok(()) => {
            report.imported = 1;
            report.document_ids.push(document_id);
        }
        Err(err) => report.errors.push(format!("{stored_path}: {err}")),
    }

    log::info!(
        "Reproceso documento: qr={} ocr={} proveedores={} err={}",
        report.qr_parsed,
        report.ocr_used,
        report.providers_created,
        report.errors.len()
    );
    Ok(report)
}

enum ImportOutcome {
    Imported(String),
    Duplicate,
}

fn import_one(
    state: &AppState,
    entity_id: &str,
    source: &Path,
    dest_root: &Path,
    report: &mut ImportReport,
) -> AppResult<ImportOutcome> {
    if !source.is_file() {
        return Err(AppError::msg("No es un archivo válido"));
    }

    let hash = file_sha256(source)?;
    let exists: bool = state.with_db(|conn| {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM documents WHERE entity_id = ?1 AND file_hash = ?2",
            params![entity_id, hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    })?;
    if exists {
        return Ok(ImportOutcome::Duplicate);
    }

    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::msg("Nombre de archivo inválido"))?
        .to_string();

    let stamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let safe_name = sanitize_filename(&file_name);
    let stored_name = format!("{stamp}_{safe_name}");
    let stored_path = dest_root.join(&stored_name);
    std::fs::copy(source, &stored_path)?;

    let entity_tax = entity_tax_id(state, entity_id);
    let extraction = extract_invoice_data_excluding(&stored_path, entity_tax.as_deref());
    if extraction.used_qr {
        report.qr_parsed += 1;
    }
    if extraction.used_ocr {
        report.ocr_used += 1;
    }
    let data = extraction.data;

    let id = new_id();
    let imported_at = now_iso();
    let mime = guess_mime(source);
    let period = data
        .period
        .clone()
        .or_else(|| guess_period_from_name(&file_name));

    let provider_id = match data.cuit.as_deref() {
        Some(cuit) => match upsert_provider(state, entity_id, cuit, data.provider_name.as_deref())
        {
            Ok((pid, created)) => {
                if created {
                    report.providers_created += 1;
                }
                Some(pid)
            }
            Err(err) => {
                log::warn!("No se pudo crear proveedor desde CUIT {}: {}", cuit, err);
                None
            }
        },
        None => None,
    };

    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO documents (
                id, entity_id, provider_id, file_name, original_path, stored_path,
                file_hash, mime_type, doc_type, number, period, issue_date, due_date,
                amount, currency, point_of_sale, fx_rate, auth_type, auth_code,
                receptor_doc_type, receptor_doc_number, notes, imported_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13,
                       ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23)",
            params![
                id,
                entity_id,
                provider_id,
                file_name,
                source.to_string_lossy().to_string(),
                stored_path.to_string_lossy().to_string(),
                hash,
                mime,
                data.doc_type.unwrap_or_else(|| "factura".into()),
                data.number,
                period,
                data.issue_date,
                data.due_date,
                data.amount,
                data.currency.unwrap_or_else(|| "ARS".into()),
                data.point_of_sale,
                data.fx_rate,
                data.auth_type,
                data.auth_code,
                data.receptor_doc_type,
                data.receptor_doc_number,
                data.source.map(|s| format!("extraído: {s}")),
                imported_at
            ],
        )?;

        conn.execute(
            "INSERT INTO payments (
                id, document_id, entity_id, status, amount_due, amount_paid, paid_at, notes, updated_at
             ) VALUES (?1, ?2, ?3, 'pending', ?4, 0, NULL, NULL, ?5)",
            params![new_id(), id, entity_id, data.amount, imported_at],
        )?;
        Ok(())
    })?;

    Ok(ImportOutcome::Imported(id))
}

fn enrich_existing_document(
    state: &AppState,
    entity_id: &str,
    doc_id: &str,
    path: &Path,
    report: &mut ImportReport,
) -> AppResult<()> {
    let entity_tax = entity_tax_id(state, entity_id);
    let extraction = extract_invoice_data_excluding(path, entity_tax.as_deref());
    if extraction.used_qr {
        report.qr_parsed += 1;
    }
    if extraction.used_ocr {
        report.ocr_used += 1;
    }
    apply_invoice_data(state, entity_id, doc_id, extraction.data, report)
}

/// Enriquece un documento con texto extraído en el frontend (PDF.js),
/// útil cuando lopdf no decodifica fuentes custom (ej. AySA).
#[tauri::command]
pub fn enrich_document_from_text(
    document_id: String,
    text: String,
    state: State<'_, AppState>,
) -> AppResult<Document> {
    if text.trim().len() < 20 {
        return Err(AppError::msg("Texto insuficiente para enriquecer"));
    }
    let (entity_id,): (String,) = state.with_db(|conn| {
        conn.query_row(
            "SELECT entity_id FROM documents WHERE id = ?1",
            params![document_id],
            |row| Ok((row.get(0)?,)),
        )
        .map_err(|_| AppError::msg("Documento no encontrado"))
    })?;

    let entity_tax = entity_tax_id(&state, &entity_id);
    let mut data = parse_invoice_text(&text, entity_tax.as_deref());
    if data.source.is_none() {
        data.source = Some("pdfjs".into());
    } else {
        data.source = Some(format!("pdfjs,{}", data.source.unwrap_or_default()));
    }

    let mut report = ImportReport {
        imported: 0,
        duplicates: 0,
        errors: Vec::new(),
        document_ids: Vec::new(),
        qr_parsed: 0,
        ocr_used: 0,
        providers_created: 0,
    };
    apply_invoice_data(&state, &entity_id, &document_id, data, &mut report)?;
    log::info!(
        "Enriquecido {} desde texto PDF.js (proveedores={})",
        document_id,
        report.providers_created
    );
    crate::commands::documents::get_document(document_id, state)
}

fn apply_invoice_data(
    state: &AppState,
    entity_id: &str,
    doc_id: &str,
    data: InvoiceData,
    report: &mut ImportReport,
) -> AppResult<()> {
    let provider_id = match data.cuit.as_deref() {
        Some(cuit) => match upsert_provider(state, entity_id, cuit, data.provider_name.as_deref())
        {
            Ok((pid, created)) => {
                if created {
                    report.providers_created += 1;
                }
                Some(pid)
            }
            Err(err) => {
                log::warn!("Reproceso: proveedor CUIT {} falló: {}", cuit, err);
                None
            }
        },
        None => None,
    };

    state.with_db(|conn| {
        // Solo completa campos vacíos
        // Monto: completar vacío o corregir parseos tipo 2.97 → 2970
        let current_amount: Option<f64> = conn
            .query_row(
                "SELECT amount FROM documents WHERE id = ?1",
                params![doc_id],
                |row| row.get(0),
            )
            .ok()
            .flatten();
        let amount_to_set = match (current_amount, data.amount) {
            (_, None) => None,
            (None, Some(n)) => Some(n),
            (Some(old), Some(new)) if old <= 0.0 => Some(new),
            (Some(old), Some(new)) => {
                let ratio = new / old;
                if (900.0..=1100.0).contains(&ratio) || (90.0..=110.0).contains(&ratio) {
                    Some(new)
                } else {
                    None // conservar monto ya cargado (p.ej. del QR)
                }
            }
        };

        conn.execute(
            "UPDATE documents SET
                provider_id = COALESCE(provider_id, ?1),
                number = COALESCE(NULLIF(number, ''), ?2),
                period = COALESCE(NULLIF(period, ''), ?3),
                issue_date = COALESCE(NULLIF(issue_date, ''), ?4),
                due_date = COALESCE(NULLIF(due_date, ''), ?5),
                amount = CASE WHEN ?6 IS NOT NULL THEN ?6 ELSE amount END,
                currency = COALESCE(NULLIF(currency, ''), ?7),
                doc_type = COALESCE(NULLIF(doc_type, ''), ?8),
                point_of_sale = COALESCE(point_of_sale, ?9),
                fx_rate = COALESCE(fx_rate, ?10),
                auth_type = COALESCE(NULLIF(auth_type, ''), ?11),
                auth_code = COALESCE(NULLIF(auth_code, ''), ?12),
                receptor_doc_type = COALESCE(receptor_doc_type, ?13),
                receptor_doc_number = COALESCE(NULLIF(receptor_doc_number, ''), ?14),
                notes = CASE
                    WHEN notes IS NULL OR notes = '' THEN ?15
                    ELSE notes
                END
             WHERE id = ?16",
            params![
                provider_id,
                data.number,
                data.period,
                data.issue_date,
                data.due_date,
                amount_to_set,
                data.currency.unwrap_or_else(|| "ARS".into()),
                data.doc_type.unwrap_or_else(|| "factura".into()),
                data.point_of_sale,
                data.fx_rate,
                data.auth_type,
                data.auth_code,
                data.receptor_doc_type,
                data.receptor_doc_number,
                data.source.map(|s| format!("extraído: {s}")),
                doc_id
            ],
        )?;

        if let Some(amount) = amount_to_set {
            conn.execute(
                "UPDATE payments SET amount_due = ?1 WHERE document_id = ?2",
                params![amount, doc_id],
            )?;
        } else if let Some(amount) = data.amount {
            conn.execute(
                "UPDATE payments SET amount_due = COALESCE(amount_due, ?1) WHERE document_id = ?2",
                params![amount, doc_id],
            )?;
        }
        Ok(())
    })
}

fn upsert_provider(
    state: &AppState,
    entity_id: &str,
    cuit_raw: &str,
    name_hint: Option<&str>,
) -> AppResult<(String, bool)> {
    let cuit = format_cuit(cuit_raw);
    let key = normalize_cuit_key(&cuit);
    if key.len() != 11 {
        return Err(AppError::msg("CUIT inválido"));
    }

    state.with_db(|conn| {
        if let Some(existing) = find_provider_by_cuit(conn, entity_id, &key)? {
            // Actualizar nombre si estaba genérico y ahora hay hint
            if let Some(name) = name_hint {
                if !name.trim().is_empty() {
                    let current: String = conn.query_row(
                        "SELECT name FROM providers WHERE id = ?1",
                        params![existing],
                        |r| r.get(0),
                    )?;
                    if current.starts_with("Proveedor ") || current.starts_with("CUIT ") {
                        conn.execute(
                            "UPDATE providers SET name = ?1 WHERE id = ?2",
                            params![name.trim(), existing],
                        )?;
                    }
                }
            }
            return Ok((existing, false));
        }

        let id = new_id();
        let name = name_hint
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Proveedor {cuit}"));
        conn.execute(
            "INSERT INTO providers (id, entity_id, name, cuit, alias, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, NULL, 'Creado automáticamente desde factura', ?5)",
            params![id, entity_id, name, cuit, now_iso()],
        )?;
        log::info!("Proveedor creado automáticamente: {} ({})", name, cuit);
        Ok((id, true))
    })
}

fn find_provider_by_cuit(
    conn: &Connection,
    entity_id: &str,
    cuit_digits: &str,
) -> AppResult<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT id, cuit FROM providers WHERE entity_id = ?1 AND cuit IS NOT NULL AND cuit != ''",
    )?;
    let rows = stmt.query_map(params![entity_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (id, cuit) = row?;
        if normalize_cuit_key(&cuit) == cuit_digits {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

fn entity_tax_id(state: &AppState, entity_id: &str) -> Option<String> {
    state
        .with_db(|conn| {
            let tax: Option<String> = conn
                .query_row(
                    "SELECT tax_id FROM entities WHERE id = ?1",
                    params![entity_id],
                    |row| row.get(0),
                )
                .unwrap_or(None);
            Ok(tax)
        })
        .ok()
        .flatten()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

fn guess_period_from_name(name: &str) -> Option<String> {
    let digits: String = name.chars().filter(|c| c.is_ascii_digit()).collect();
    if let Some(idx) = name.find('-') {
        if idx >= 4 {
            let slice = &name[idx.saturating_sub(4)..];
            if slice.len() >= 7 {
                let y = &slice[0..4];
                let m = &slice[5..7];
                if y.chars().all(|c| c.is_ascii_digit())
                    && m.chars().all(|c| c.is_ascii_digit())
                    && slice.as_bytes().get(4) == Some(&b'-')
                {
                    return Some(format!("{y}-{m}"));
                }
            }
        }
    }
    if digits.len() >= 6 {
        let y = &digits[0..4];
        let m = &digits[4..6];
        if let Ok(month) = m.parse::<u32>() {
            if (1..=12).contains(&month) {
                return Some(format!("{y}-{m}"));
            }
        }
    }
    None
}
