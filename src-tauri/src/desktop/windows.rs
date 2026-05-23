use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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
