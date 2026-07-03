pub mod logging;
pub mod oci;
pub mod storage;
pub mod services;
pub mod tarball;
pub mod commands;
pub mod auth_error;
pub mod error;
pub mod pull_error;
pub mod registry_error;
pub mod storage_error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;
    use crate::storage::sqlite::SqliteHandle;
    use std::path::PathBuf;
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            let path = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from(".")).join("imagevue.db");
            let handle = SqliteHandle::open(&path).expect("open db");
            app.manage(handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection::list_connections,
            commands::connection::get_connection,
            commands::connection::create_connection,
            commands::connection::delete_connection,
            commands::connection::test_connection,
            commands::catalog::list_repositories,
            commands::tag::list_tags,
            commands::manifest::get_manifest,
            commands::manifest::get_image_config,
            commands::pull::start_pull,
            commands::auth::begin_oauth,
            commands::auth::complete_oauth,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
