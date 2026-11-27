use crate::error::AppResult;
use crate::models::{Entity, NewEntity};
use crate::state::AppState;
use crate::util::{new_id, now_iso};
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_entities(state: State<'_, AppState>) -> AppResult<Vec<Entity>> {
    state.with_db(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, kind, tax_id, created_at, updated_at
             FROM entities ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Entity {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                tax_id: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }
        log::info!("Listado de entidades: {}", entities.len());
        Ok(entities)
    })
}

#[tauri::command]
pub fn create_entity(payload: NewEntity, state: State<'_, AppState>) -> AppResult<Entity> {
    let kind = if payload.kind == "company" {
        "company"
    } else {
        "personal"
    };
    if payload.name.trim().is_empty() {
        return Err(crate::error::AppError::msg("El nombre es obligatorio"));
    }

    let entity = Entity {
        id: new_id(),
        name: payload.name.trim().to_string(),
        kind: kind.to_string(),
        tax_id: payload
            .tax_id
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty()),
        created_at: now_iso(),
        updated_at: now_iso(),
    };

    state.with_db(|conn| {
        conn.execute(
            "INSERT INTO entities (id, name, kind, tax_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entity.id,
                entity.name,
                entity.kind,
                entity.tax_id,
                entity.created_at,
                entity.updated_at
            ],
        )?;
        Ok(())
    })?;

    std::fs::create_dir_all(state.entity_docs_dir(&entity.id)?)?;
    log::info!("Entidad creada: {} ({})", entity.name, entity.id);
    Ok(entity)
}

#[tauri::command]
pub fn update_entity(
    id: String,
    payload: NewEntity,
    state: State<'_, AppState>,
) -> AppResult<Entity> {
    let kind = if payload.kind == "company" {
        "company"
    } else {
        "personal"
    };
    if payload.name.trim().is_empty() {
        return Err(crate::error::AppError::msg("El nombre es obligatorio"));
    }
    let updated_at = now_iso();
    let tax_id = payload
        .tax_id
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    state.with_db(|conn| {
        let changed = conn.execute(
            "UPDATE entities SET name = ?1, kind = ?2, tax_id = ?3, updated_at = ?4 WHERE id = ?5",
            params![payload.name.trim(), kind, tax_id, updated_at, id],
        )?;
        if changed == 0 {
            return Err(crate::error::AppError::msg("Entidad no encontrada"));
        }
        Ok(())
    })?;

    list_entities(state)?
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| crate::error::AppError::msg("Entidad no encontrada"))
}

#[tauri::command]
pub fn delete_entity(id: String, state: State<'_, AppState>) -> AppResult<()> {
    state.with_db(|conn| {
        let changed = conn.execute("DELETE FROM entities WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(crate::error::AppError::msg("Entidad no encontrada"));
        }
        Ok(())
    })?;
    log::info!("Entidad eliminada: {}", id);
    Ok(())
}
