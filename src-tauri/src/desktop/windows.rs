use crate::error::{AppError, AppResult};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const MASCOT_WINDOW_SIZE: f64 = 80.0;
const MASCOT_WINDOW_MARGIN: f64 = 40.0;
const LIVE_CAPTION_WINDOW_WIDTH: f64 = 420.0;
const LIVE_CAPTION_WINDOW_HEIGHT: f64 = 96.0;
const LIVE_CAPTION_BOTTOM_MARGIN: f64 = 56.0;
const LIVE_CAPTION_MIN_MARGIN: f64 = 24.0;

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
        let _ = window.set_always_on_top(true);
        position_mascot_window(&window);
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

    position_mascot_window(&window);
    let _ = window.show();

    Ok(())
}

pub fn spawn_live_caption_window(app: &AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("live-caption") {
        let _ = window.set_always_on_top(true);
        position_live_caption_window(&window);
        window.show().map_err(|error| AppError::Window(error.to_string()))?;
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        "live-caption",
        WebviewUrl::App("/?window=live-caption".into()),
    )
    .title("FlowType Live Caption")
    .inner_size(LIVE_CAPTION_WINDOW_WIDTH, LIVE_CAPTION_WINDOW_HEIGHT)
    .resizable(false)
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .build()
    .map_err(|error| AppError::Window(error.to_string()))?;

    position_live_caption_window(&window);
    let _ = window.show();

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

fn position_mascot_window(window: &tauri::WebviewWindow) {
    if let Some((origin_x, origin_y, logical_width, logical_height)) = current_monitor_frame(window) {
        let x = origin_x + logical_width - MASCOT_WINDOW_SIZE - MASCOT_WINDOW_MARGIN;
        let y = origin_y + logical_height - MASCOT_WINDOW_SIZE - MASCOT_WINDOW_MARGIN;
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    }
}

fn position_live_caption_window(window: &tauri::WebviewWindow) {
    if let Some((origin_x, origin_y, logical_width, logical_height)) = current_monitor_frame(window) {
        let x = origin_x + (logical_width - LIVE_CAPTION_WINDOW_WIDTH) / 2.0;
        let min_y = origin_y + LIVE_CAPTION_MIN_MARGIN;
        let preferred_y =
            origin_y + logical_height - LIVE_CAPTION_WINDOW_HEIGHT - LIVE_CAPTION_BOTTOM_MARGIN;
        let max_y = origin_y + logical_height - LIVE_CAPTION_WINDOW_HEIGHT - LIVE_CAPTION_MIN_MARGIN;
        let y = preferred_y.clamp(min_y, max_y);
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    }
}

fn current_monitor_frame(window: &tauri::WebviewWindow) -> Option<(f64, f64, f64, f64)> {
    let monitor = window.current_monitor().ok().flatten()?;
    let size = monitor.size();
    let position = monitor.position();
    let scale_factor = monitor.scale_factor();
    let origin_x = (position.x as f64) / scale_factor;
    let origin_y = (position.y as f64) / scale_factor;
    let logical_width = (size.width as f64) / scale_factor;
    let logical_height = (size.height as f64) / scale_factor;
    Some((origin_x, origin_y, logical_width, logical_height))
}
