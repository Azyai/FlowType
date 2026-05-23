use crate::{
    settings::{AppSettings, OutputStyle},
    storage::DatabaseHealth,
    error::{AppError, AppResult, CommandResult, ErrorResponse},
    app::AppState,
    updates::{self, UpdateCheckResult},
    desktop::windows,
};
use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub app_version: String,
    pub paused: bool,
    pub current_mode: String,
    pub tray_available: bool,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CommandResult<AppSettings> {
    into_command(state.settings())
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> CommandResult<AppSettings> {
    into_command(state.save_settings(settings))
}

#[tauri::command]
pub fn reset_settings(state: State<AppState>) -> CommandResult<AppSettings> {
    into_command(state.reset_settings())
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
