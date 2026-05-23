use crate::{
    asr::{self, AsrServiceCheckResult, AsrServiceConfig},
    settings::{AppSettings, OutputStyle},
    storage::DatabaseHealth,
    error::{AppError, AppResult, CommandResult, ErrorResponse},
    app::AppState,
    updates::{self, UpdateCheckResult},
    desktop::{tray, windows},
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State, Manager, Emitter};
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub app_version: String,
    pub paused: bool,
    pub current_mode: String,
    pub tray_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClearHistoryResult {
    pub deleted_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AsrServiceConfigInput {
    pub asr_service_mode: crate::settings::AsrServiceMode,
    pub iflytek_app_id: String,
    pub iflytek_api_key: String,
    pub iflytek_api_secret: String,
    pub iflytek_language: crate::settings::IflytekLanguage,
    pub iflytek_mixed_language: bool,
    pub iflytek_timeout_ms: u64,
    pub iflytek_retry_count: u8,
}

#[tauri::command]
pub fn toggle_recording(app: AppHandle) -> CommandResult<()> {
    let currently_recording = crate::desktop::hotkey::IS_RECORDING.load(std::sync::atomic::Ordering::SeqCst);
    if currently_recording {
        crate::desktop::hotkey::IS_RECORDING.store(false, std::sync::atomic::Ordering::SeqCst);
        let _ = app.emit("status_changed", "Processing");
        
        let app_handle_clone = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let _ = app_handle_clone.emit("status_changed", "Injecting");
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = app_handle_clone.emit("status_changed", "Idle");
        });
    } else {
        crate::desktop::hotkey::IS_RECORDING.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = app.emit("status_changed", "Listening");
    }
    
    Ok(().into())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CommandResult<AppSettings> {
    into_command(state.settings())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> CommandResult<AppSettings> {
    into_command(save_settings_and_refresh_tray(&app, &state, settings))
}

#[tauri::command]
pub fn reset_settings(app: AppHandle, state: State<AppState>) -> CommandResult<AppSettings> {
    into_command(reset_settings_and_refresh_tray(&app, &state))
}

#[tauri::command]
pub fn get_app_status(state: State<AppState>) -> CommandResult<AppStatus> {
    into_command(app_status(&state))
}

#[tauri::command]
pub fn set_autostart(
    app: AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> CommandResult<AppSettings> {
    into_command(apply_autostart(&app, &state, enabled))
}

#[tauri::command]
pub fn get_database_health(state: State<AppState>) -> CommandResult<DatabaseHealth> {
    Ok(state.database_health())
}

#[tauri::command]
pub fn check_update(state: State<AppState>) -> CommandResult<UpdateCheckResult> {
    into_command(update_check(&state))
}

#[tauri::command]
pub fn get_asr_service_config(state: State<AppState>) -> CommandResult<AsrServiceConfig> {
    into_command(state.settings().map(|settings| asr::service_config(&settings)))
}

#[tauri::command]
pub fn save_asr_service_config(
    app: AppHandle,
    state: State<AppState>,
    config: AsrServiceConfigInput,
) -> CommandResult<AppSettings> {
    into_command(save_asr_config_and_refresh_tray(&app, &state, config))
}

#[tauri::command]
pub fn check_asr_service(state: State<AppState>) -> CommandResult<AsrServiceCheckResult> {
    into_command(state.settings().map(|settings| asr::check_service(&settings)))
}

#[tauri::command]
pub fn clear_history(state: State<AppState>) -> CommandResult<ClearHistoryResult> {
    into_command(state.clear_history().map(|deleted_count| ClearHistoryResult { deleted_count }))
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle) -> CommandResult<()> {
    into_command(windows::show_main_window(&app))
}

#[tauri::command]
pub async fn open_about_window(app: AppHandle) -> CommandResult<()> {
    into_command(windows::show_about_window(&app))
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> CommandResult<()> {
    app.exit(0);
    Ok(())
}

pub fn set_output_style(state: &AppState, output_style: OutputStyle) -> CommandResult<AppSettings> {
    into_command(state.update_output_style(output_style))
}

fn save_settings_and_refresh_tray(
    app: &AppHandle,
    state: &AppState,
    settings: AppSettings,
) -> AppResult<AppSettings> {
    let saved = state.save_settings(settings)?;
    if let Err(error) = tray::refresh(app) {
        log::warn!("failed to refresh tray after saving settings: {error:?}");
    }
    Ok(saved)
}

fn reset_settings_and_refresh_tray(app: &AppHandle, state: &AppState) -> AppResult<AppSettings> {
    let saved = state.reset_settings()?;
    if let Err(error) = tray::refresh(app) {
        log::warn!("failed to refresh tray after resetting settings: {error:?}");
    }
    Ok(saved)
}

fn save_asr_config_and_refresh_tray(
    app: &AppHandle,
    state: &AppState,
    config: AsrServiceConfigInput,
) -> AppResult<AppSettings> {
    let mut settings = state.settings()?;
    settings.asr_service_mode = config.asr_service_mode;
    settings.iflytek_app_id = config.iflytek_app_id;
    settings.iflytek_api_key = config.iflytek_api_key;
    settings.iflytek_api_secret = config.iflytek_api_secret;
    settings.iflytek_language = config.iflytek_language;
    settings.iflytek_mixed_language = config.iflytek_mixed_language;
    settings.iflytek_timeout_ms = config.iflytek_timeout_ms;
    settings.iflytek_retry_count = config.iflytek_retry_count;
    save_settings_and_refresh_tray(app, state, settings)
}

pub fn app_status(state: &AppState) -> AppResult<AppStatus> {
    let settings = state.settings()?;
    Ok(AppStatus {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        paused: state.paused(),
        current_mode: format!("{:?}", settings.output_style).to_lowercase(),
        tray_available: true,
    })
}

fn apply_autostart(
    app: &AppHandle,
    state: &AppState,
    enabled: bool,
) -> AppResult<AppSettings> {
    if enabled {
        app.autolaunch()
            .enable()
            .map_err(|error| AppError::Autostart(error.to_string()))?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| AppError::Autostart(error.to_string()))?;
    }

    let mut settings = state.settings()?;
    settings.auto_start = enabled;
    state.save_settings(settings)
}

fn update_check(state: &AppState) -> AppResult<UpdateCheckResult> {
    let settings = state.settings()?;
    updates::check_for_update(&settings, env!("CARGO_PKG_VERSION"))
}

fn into_command<T>(result: AppResult<T>) -> CommandResult<T> {
    result.map_err(|error| {
        let response: ErrorResponse = error.into();
        log::error!("{}: {}", response.code, response.message);
        response
    })
}
