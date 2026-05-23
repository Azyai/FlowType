mod app;
mod asr;
mod commands;
mod desktop;
mod error;
mod settings;
mod storage;
mod updates;
mod voice;

use app::AppState;
use commands::{
    check_asr_service, check_update, clear_history, get_app_status, get_asr_service_config,
    get_database_health, get_settings, open_about_window, open_settings_window, quit_app,
    reset_settings, save_asr_service_config, save_settings, set_autostart, start_voice_input,
    stop_voice_input, cancel_voice_input, get_voice_status, show_mascot_window, hide_mascot_window,
    set_output_mode, toggle_recording,
};
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

            app.manage(AppState::new(config_store, settings.clone(), database));
            tray::create(app)?;
            
            if settings.show_floating_window {
                if let Err(error) = desktop::windows::spawn_mascot_window(app.handle()) {
                    log::error!("failed to create mascot window: {error}");
                }
            }

            desktop::hotkey::start_hotkey_listener(app.handle().clone());
            
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
            get_asr_service_config,
            save_asr_service_config,
            check_asr_service,
            get_app_status,
            set_autostart,
            get_database_health,
            clear_history,
            check_update,
            start_voice_input,
            stop_voice_input,
            cancel_voice_input,
            get_voice_status,
            show_mascot_window,
            hide_mascot_window,
            set_output_mode,
            open_settings_window,
            open_about_window,
            quit_app,
            toggle_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running FlowType");
}
