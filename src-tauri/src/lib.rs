mod arca;
mod commands;
mod db;
mod error;
mod extract;
mod logutil;
mod models;
mod state;
mod util;

#[cfg(test)]
mod e2e_user_sim;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_dialog::init())
            .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("No se pudo resolver app_data_dir: {e}"))?;
            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("No se pudo crear data_dir: {e}"))?;

            let _ = logutil::init(&data_dir);

            let app_state = AppState::init(data_dir).map_err(|e| e.to_string())?;
            app.manage(app_state);
            log::info!("Folio listo");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::entities::list_entities,
            commands::entities::create_entity,
            commands::entities::update_entity,
            commands::entities::delete_entity,
            commands::providers::list_providers,
            commands::providers::create_provider,
            commands::providers::update_provider,
            commands::providers::delete_provider,
            commands::documents::get_document,
            commands::documents::update_document_meta,
            commands::documents::delete_document,
            commands::documents::list_recent_documents,
            commands::import::import_files,
            commands::import::reprocess_documents,
            commands::import::reprocess_document,
            commands::import::enrich_document_from_text,
            commands::search::search_documents,
            commands::search::export_search_csv,
            commands::payments::list_payments,
            commands::payments::update_payment,
            commands::payments::list_agenda,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            commands::config::get_app_info,
            commands::config::get_paths,
            commands::config::update_paths,
            commands::config::get_setting,
            commands::config::set_setting,
            commands::config::get_dashboard_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Folio");
}
