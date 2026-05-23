mod app;
mod commands;
mod desktop;
mod error;
mod settings;
mod storage;
mod updates;

use commands::{
    check_update, get_app_status, get_database_health, get_settings, open_about_window,
    open_settings_window, quit_app, reset_settings, save_settings, set_autostart,
};
use app::AppState;
use desktop::tray;
use settings::ConfigStore;
use storage::Database;
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let config_store = ConfigStore::new(app_data_dir.join("settings.json"));
            let settings = config_store.load()?;
            let database = Database::open(app_data_dir.join("app.db"))?;

            app.manage(AppState::new(config_store, settings, database));
            tray::create(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Err(error) = window.hide() {
                        log::error!("failed to hide main window: {error}");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            reset_settings,
            get_app_status,
            set_autostart,
            get_database_health,
            check_update,
            open_settings_window,
            open_about_window,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlowType");
}
