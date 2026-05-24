use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const MASCOT_WINDOW_SIZE: f64 = 80.0;
const MASCOT_WINDOW_MARGIN: f64 = 40.0;

pub fn show_main_window(app: &AppHandle) -> AppResult<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Err(AppError::Window("main window is not registered".to_string()));
    };

    window.show().map_err(|error| AppError::Window(error.to_string()))?;
    window
        .set_focus()
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok(())
}

pub fn spawn_mascot_window(app: &AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("mascot") {
        window.show().map_err(|error| AppError::Window(error.to_string()))?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, "mascot", WebviewUrl::App("/?window=mascot".into()))
        .title("FlowType Mascot")
        .inner_size(MASCOT_WINDOW_SIZE, MASCOT_WINDOW_SIZE)
        .resizable(false)
        .transparent(true)
        .decorations(false)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|error| AppError::Window(error.to_string()))?;

    // Default position to bottom right
    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let logical_width = (size.width as f64) / scale_factor;
        let logical_height = (size.height as f64) / scale_factor;
        
        let x = logical_width - MASCOT_WINDOW_SIZE - MASCOT_WINDOW_MARGIN;
        let y = logical_height - MASCOT_WINDOW_SIZE - MASCOT_WINDOW_MARGIN;
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    }

    Ok(())
}

pub fn show_about_window(app: &AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("about") {
        window.show().map_err(|error| AppError::Window(error.to_string()))?;
        window
            .set_focus()
            .map_err(|error| AppError::Window(error.to_string()))?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "about", WebviewUrl::App("index.html".into()))
        .title("About FlowType")
        .inner_size(520.0, 460.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok(())
}
