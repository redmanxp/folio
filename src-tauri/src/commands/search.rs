use crate::commands::documents::{map_document, DOC_SELECT};
use crate::db;
use crate::error::AppResult;
use crate::models::{SearchFilters, SearchResult};
use crate::state::AppState;
use rusqlite::params_from_iter;
use tauri::State;

#[tauri::command]
pub fn search_documents(
    filters: SearchFilters,
    state: State<'_, AppState>,
) -> AppResult<SearchResult> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &filters.entity_id)?;

        let mut sql = format!("{DOC_SELECT} WHERE d.entity_id = ?1");
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(filters.entity_id.clone())];

        if let Some(q) = filters.query.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
            sql.push_str(
                " AND (
                    d.file_name LIKE ? OR
                    IFNULL(d.number,'') LIKE ? OR
                    IFNULL(d.notes,'') LIKE ? OR
                    IFNULL(p.name,'') LIKE ? OR
                    IFNULL(p.cuit,'') LIKE ? OR
                    IFNULL(p.alias,'') LIKE ?
                  )",
            );
            let like = format!("%{q}%");
            for _ in 0..6 {
                values.push(Box::new(like.clone()));
            }
        }

        if let Some(provider) = filters
            .provider
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND (IFNULL(p.name,'') LIKE ? OR IFNULL(p.alias,'') LIKE ?)");
            let like = format!("%{provider}%");
            values.push(Box::new(like.clone()));
            values.push(Box::new(like));
        }

        if let Some(cuit) = filters
            .cuit
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(p.cuit,'') LIKE ?");
            values.push(Box::new(format!("%{cuit}%")));
        }

        if let Some(number) = filters
            .number
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(d.number,'') LIKE ?");
            values.push(Box::new(format!("%{number}%")));
        }

        if let Some(period) = filters
            .period
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(d.period,'') = ?");
            values.push(Box::new(period));
        }

        if let Some(status) = filters
            .payment_status
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(pay.status,'') = ?");
            values.push(Box::new(status));
        }

        if let Some(from) = filters
            .from_date
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(d.issue_date, d.imported_at) >= ?");
            values.push(Box::new(from));
        }

        if let Some(to) = filters
            .to_date
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            sql.push_str(" AND IFNULL(d.issue_date, d.imported_at) <= ?");
            values.push(Box::new(to));
        }

        sql.push_str(" ORDER BY IFNULL(d.issue_date, d.imported_at) DESC, d.imported_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(values.iter().map(|v| &**v)), map_document)?;
        let mut documents = Vec::new();
        for row in rows {
            documents.push(row?);
        }

        let total_amount = documents.iter().filter_map(|d| d.amount).sum();
        let count = documents.len();
        log::info!("Búsqueda entidad {}: {} resultados", filters.entity_id, count);
        Ok(SearchResult {
            documents,
            total_amount,
            count,
        })
    })
}

#[tauri::command]
pub fn export_search_csv(
    filters: SearchFilters,
    target_path: String,
    state: State<'_, AppState>,
) -> AppResult<String> {
    let result = search_documents(filters, state)?;
    let mut writer = csv::Writer::from_path(&target_path)?;
    writer.write_record([
        "archivo",
        "proveedor",
        "cuit",
        "numero",
        "periodo",
        "fecha",
        "vencimiento",
        "importe",
        "moneda",
        "estado_pago",
        "ruta",
    ])?;

    for doc in result.documents {
        writer.write_record([
            csv_safe(&doc.file_name),
            csv_safe(&doc.provider_name.unwrap_or_default()),
            String::new(),
            csv_safe(&doc.number.unwrap_or_default()),
            csv_safe(&doc.period.unwrap_or_default()),
            csv_safe(&doc.issue_date.unwrap_or_default()),
            csv_safe(&doc.due_date.unwrap_or_default()),
            doc.amount.map(|a| a.to_string()).unwrap_or_default(),
            csv_safe(&doc.currency),
            csv_safe(&doc.payment_status.unwrap_or_default()),
            csv_safe(&doc.stored_path),
        ])?;
    }
    writer.flush()?;
    log::info!("Exportación CSV completada");
    Ok(target_path)
}

/// Evita que Excel/LibreOffice interprete celdas como fórmulas.
fn csv_safe(value: &str) -> String {
    let trimmed = value.trim_start_matches([' ', '\t', '\r', '\n']);
    let dangerous = trimmed.starts_with(['=', '+', '-', '@', '\t', '\r']);
    if dangerous {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod csv_tests {
    use super::csv_safe;

    #[test]
    fn sanitizes_formula_prefixes() {
        assert_eq!(csv_safe("=1+1"), "'=1+1");
        assert_eq!(csv_safe("+cmd"), "'+cmd");
        assert_eq!(csv_safe("-2"), "'-2");
        assert_eq!(csv_safe("@sum"), "'@sum");
        assert_eq!(csv_safe("  =x"), "'  =x");
        assert_eq!(csv_safe("Norte Energia"), "Norte Energia");
    }
}
