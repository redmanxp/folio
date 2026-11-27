use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{NewProvider, Provider, UpdateProvider};
use crate::state::AppState;
use crate::util::{new_id, now_iso};
use rusqlite::params;
use tauri::State;

fn opt_trim(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn map_provider(row: &rusqlite::Row<'_>) -> rusqlite::Result<Provider> {
    Ok(Provider {
        id: row.get(0)?,
        entity_id: row.get(1)?,
        name: row.get(2)?,
        cuit: row.get(3)?,
        alias: row.get(4)?,
        cbu: row.get(5)?,
        email: row.get(6)?,
        phone: row.get(7)?,
        notes: row.get(8)?,
        created_at: row.get(9)?,
    })
}

const PROVIDER_SELECT: &str = "
    SELECT id, entity_id, name, cuit, alias, cbu, email, phone, notes, created_at
    FROM providers
";

#[tauri::command]
pub fn list_providers(entity_id: String, state: State<'_, AppState>) -> AppResult<Vec<Provider>> {
    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &entity_id)?;
        let sql = format!("{PROVIDER_SELECT} WHERE entity_id = ?1 ORDER BY name COLLATE NOCASE");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![entity_id], map_provider)?;
        let mut items = Vec::new();
        for row in rows {
            items.push(row?);
        }
        Ok(items)
    })
}

#[tauri::command]
pub fn create_provider(payload: NewProvider, state: State<'_, AppState>) -> AppResult<Provider> {
    if payload.name.trim().is_empty() {
        return Err(AppError::msg("El nombre del proveedor es obligatorio"));
    }
    let provider = Provider {
        id: new_id(),
        entity_id: payload.entity_id.clone(),
        name: payload.name.trim().to_string(),
        cuit: opt_trim(payload.cuit),
        alias: opt_trim(payload.alias),
        cbu: opt_trim(payload.cbu),
        email: opt_trim(payload.email),
        phone: opt_trim(payload.phone),
        notes: opt_trim(payload.notes),
        created_at: now_iso(),
    };

    state.with_db(|conn| {
        db::ensure_entity_exists(conn, &provider.entity_id)?;
        conn.execute(
            "INSERT INTO providers (
                id, entity_id, name, cuit, alias, cbu, email, phone, notes, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                provider.id,
                provider.entity_id,
                provider.name,
                provider.cuit,
                provider.alias,
                provider.cbu,
                provider.email,
                provider.phone,
                provider.notes,
                provider.created_at
            ],
        )?;
        Ok(())
    })?;

    log::info!("Proveedor creado: {}", provider.name);
    Ok(provider)
}

#[tauri::command]
pub fn update_provider(payload: UpdateProvider, state: State<'_, AppState>) -> AppResult<Provider> {
    if payload.name.trim().is_empty() {
        return Err(AppError::msg("El nombre del proveedor es obligatorio"));
    }
    let name = payload.name.trim().to_string();
    let cuit = opt_trim(payload.cuit);
    let alias = opt_trim(payload.alias);
    let cbu = opt_trim(payload.cbu);
    let email = opt_trim(payload.email);
    let phone = opt_trim(payload.phone);
    let notes = opt_trim(payload.notes);

    state.with_db(|conn| {
        let changed = conn.execute(
            "UPDATE providers SET
                name = ?1, cuit = ?2, alias = ?3, cbu = ?4, email = ?5, phone = ?6, notes = ?7
             WHERE id = ?8",
            params![name, cuit, alias, cbu, email, phone, notes, payload.id],
        )?;
        if changed == 0 {
            return Err(AppError::msg("Proveedor no encontrado"));
        }

        let sql = format!("{PROVIDER_SELECT} WHERE id = ?1");
        conn.query_row(&sql, params![payload.id], map_provider)
            .map_err(AppError::from)
    })
}

#[tauri::command]
pub fn delete_provider(id: String, state: State<'_, AppState>) -> AppResult<()> {
    state.with_db(|conn| {
        let changed = conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(AppError::msg("Proveedor no encontrado"));
        }
        Ok(())
    })
}
