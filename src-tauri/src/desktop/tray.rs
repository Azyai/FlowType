use crate::{
    commands,
    settings::OutputStyle,
    error::{AppError, AppResult},
    app::AppState,
    desktop::windows,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager,
};

pub fn create(app: &App) -> tauri::Result<()> {
    let open_settings = MenuItem::with_id(app, "open_settings", "Open Settings", true, None::<&str>)?;
    let pause_voice = MenuItem::with_id(app, "pause_voice", "Pause Voice Input", true, None::<&str>)?;
    let mode_raw = MenuItem::with_id(app, "mode_raw", "Mode: Raw", true, None::<&str>)?;
    let mode_clean = MenuItem::with_id(app, "mode_clean", "Mode: Clean", true, None::<&str>)?;
    let mode_formal = MenuItem::with_id(app, "mode_formal", "Mode: Formal", true, None::<&str>)?;
    let check_microphone =
        MenuItem::with_id(app, "check_microphone", "Check Microphone", true, None::<&str>)?;
    let view_history = MenuItem::with_id(app, "view_history", "View History", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit FlowType", true, None::<&str>)?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let separator_three = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &open_settings,
            &pause_voice,
            &separator_one,
            &mode_raw,
            &mode_clean,
            &mode_formal,
            &separator_two,
            &check_microphone,
            &view_history,
            &separator_three,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(create_icon())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if let Err(error) = handle_menu_event(app, event.id().as_ref()) {
                log::error!("tray menu failed: {error:?}");
            }
        })
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, id: &str) -> AppResult<()> {
    match id {
        "open_settings" => windows::show_main_window(app),
        "pause_voice" => {
            if let Some(state) = app.try_state::<AppState>() {
                let paused = state.toggle_paused();
                log::info!("voice input paused: {paused}");
            }
            Ok(())
        }
        "mode_raw" => set_mode(app, OutputStyle::Raw),
        "mode_clean" => set_mode(app, OutputStyle::Clean),
        "mode_formal" => set_mode(app, OutputStyle::Formal),
        "check_microphone" => {
            log::info!("microphone check requested; audio capture starts in Phase 2");
            windows::show_main_window(app)
        }
        "view_history" => {
            log::info!("history requested; history tables start in a later phase");
            windows::show_main_window(app)
        }
        "quit" => {
            app.exit(0);
            Ok(())
        }
        other => Err(AppError::Window(format!("unknown tray menu id: {other}"))),
    }
}

fn set_mode(app: &AppHandle, output_style: OutputStyle) -> AppResult<()> {
    let Some(state) = app.try_state::<AppState>() else {
        return Err(AppError::StateLock);
    };

    commands::set_output_style(&state, output_style).map_err(|error| {
        AppError::Window(format!("failed to update output mode: {}", error.message))
    })?;
    Ok(())
}

fn create_icon() -> Image<'static> {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let offset = (y * width + x) * 4;
            let inside = (4..28).contains(&x) && (4..28).contains(&y);
            let accent = (8..24).contains(&x) && (8..12).contains(&y)
                || (8..19).contains(&x) && (14..18).contains(&y)
                || (8..13).contains(&x) && (8..25).contains(&y);

            if inside {
                rgba[offset] = 24;
                rgba[offset + 1] = 32;
                rgba[offset + 2] = 45;
                rgba[offset + 3] = 255;
            }

            if accent {
                rgba[offset] = 113;
                rgba[offset + 1] = 230;
                rgba[offset + 2] = 188;
                rgba[offset + 3] = 255;
            }
        }
    }

    Image::new_owned(rgba, width as u32, height as u32)
}
