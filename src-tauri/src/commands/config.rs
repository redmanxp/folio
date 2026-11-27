use crate::db;
use crate::error::AppResult;
use crate::models::{AppInfo, AppPaths, DashboardStats};
use crate::state::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Folio".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        description: "Facturas, documentos y pagos. Todo en orden.".into(),
    }
}

#[tauri::command]
pub fn get_paths(state: State<'_, AppState>) -> AppResult<AppPaths> {
    state.paths_public()
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePathsPayload {
    pub documents_dir: Option<String>,
    pub backups_dir: Option<String>,
}

#[tauri::command]
pub fn update_paths(
    payload: UpdatePathsPayload,
    state: State<'_, AppState>,
) -> AppResult<AppPaths> {
    state.update_local_paths(payload.documents_dir, payload.backups_dir)
}

#[tauri::command]
pub fn get_setting(key: String, state: State<'_, AppState>) -> AppResult<Option<String>> {
    state.with_db(|conn| db::get_setting(conn, &key))
}

#[tauri::command]
pub fn set_setting(key: String, value: String, state: State<'_, AppState>) -> AppResult<()> {
    state.with_db(|conn| {
        db::set_setting(conn, &key, &value)?;
        log::info!("Setting actualizado: {}", key);
        Ok(())
    })
}

#[tauri::command]
pub fn get_dashboard_stats(
    entity_id: Option<String>,
    state: State<'_, AppState>,
) -> AppResult<DashboardStats> {
    state.with_db(|conn| {
        let entities: usize = conn.query_row("SELECT COUNT(*) FROM entities", [], |r| {
            r.get::<_, i64>(0)
        })? as usize;

        if let Some(eid) = entity_id.filter(|s| !s.is_empty()) {
            let documents: usize = conn.query_row(
                "SELECT COUNT(*) FROM documents WHERE entity_id = ?1",
                params![eid],
                |r| r.get::<_, i64>(0),
            )? as usize;
            let providers: usize = conn.query_row(
                "SELECT COUNT(*) FROM providers WHERE entity_id = ?1",
                params![eid],
                |r| r.get::<_, i64>(0),
            )? as usize;
            let pending_payments: usize = conn.query_row(
                "SELECT COUNT(*) FROM payments WHERE entity_id = ?1 AND status IN ('pending','partial')",
                params![eid],
                |r| r.get::<_, i64>(0),
            )? as usize;
            let due_soon: usize = conn.query_row(
                "SELECT COUNT(*) FROM payments pay
                 JOIN documents d ON d.id = pay.document_id
                 WHERE pay.entity_id = ?1
                   AND pay.status IN ('pending','partial')
                   AND d.due_date IS NOT NULL
                   AND d.due_date <= date('now', '+7 day')",
                params![eid],
                |r| r.get::<_, i64>(0),
            )? as usize;

            return Ok(DashboardStats {
                entities,
                documents,
                providers,
                pending_payments,
                due_soon,
            });
        }

        let documents: usize =
            conn.query_row("SELECT COUNT(*) FROM documents", [], |r| r.get::<_, i64>(0))? as usize;
        let providers: usize =
            conn.query_row("SELECT COUNT(*) FROM providers", [], |r| r.get::<_, i64>(0))? as usize;
        let pending_payments: usize = conn.query_row(
            "SELECT COUNT(*) FROM payments WHERE status IN ('pending','partial')",
            [],
            |r| r.get::<_, i64>(0),
        )? as usize;
        let due_soon: usize = conn.query_row(
            "SELECT COUNT(*) FROM payments pay
             JOIN documents d ON d.id = pay.document_id
             WHERE pay.status IN ('pending','partial')
               AND d.due_date IS NOT NULL
               AND d.due_date <= date('now', '+7 day')",
            [],
            |r| r.get::<_, i64>(0),
        )? as usize;

        Ok(DashboardStats {
            entities,
            documents,
            providers,
            pending_payments,
            due_soon,
        })
    })
}
