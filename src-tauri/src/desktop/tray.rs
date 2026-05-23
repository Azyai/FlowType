use crate::{
    app::AppState,
    commands,
    desktop::tray_i18n::{mode_label, tray_labels},
    settings::OutputStyle,
    error::{AppError, AppResult},
    desktop::windows,
};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, AppHandle, Manager,
};

pub fn create(app: &App) -> tauri::Result<()> {
    let menu = build_menu(app.app_handle())?;

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

pub fn refresh(app: &AppHandle) -> AppResult<()> {
    let menu = build_menu(app).map_err(|error| AppError::Window(error.to_string()))?;
    let Some(tray) = app.tray_by_id("main") else {
        return Err(AppError::Window("main tray is not registered".to_string()));
    };

    tray.set_menu(Some(menu))
        .map_err(|error| AppError::Window(error.to_string()))?;
    Ok(())
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let (labels, paused, output_style) = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .settings()
                .ok()
                .map(|settings| (tray_labels(&settings.locale_preference), state.paused(), settings.output_style))
        })
        .unwrap_or_else(|| {
            let labels = tray_labels(&crate::settings::LocalePreference::Auto);
            (labels, false, OutputStyle::Clean)
        });

    let pause_label = if paused {
        labels.resume_voice
    } else {
        labels.pause_voice
    };

    let open_settings = MenuItem::with_id(app, "open_settings", labels.open_settings, true, None::<&str>)?;
    let pause_voice = MenuItem::with_id(app, "pause_voice", pause_label, true, None::<&str>)?;
    let mode_raw = MenuItem::with_id(app, "mode_raw", labels.mode_raw, true, None::<&str>)?;
    let mode_clean = MenuItem::with_id(app, "mode_clean", labels.mode_clean, true, None::<&str>)?;
    let mode_formal = MenuItem::with_id(app, "mode_formal", labels.mode_formal, true, None::<&str>)?;
    let check_microphone =
        MenuItem::with_id(app, "check_microphone", labels.check_microphone, true, None::<&str>)?;
    let view_history = MenuItem::with_id(app, "view_history", labels.view_history, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    let separator_one = PredefinedMenuItem::separator(app)?;
    let separator_two = PredefinedMenuItem::separator(app)?;
    let separator_three = PredefinedMenuItem::separator(app)?;

    log::debug!("building tray menu with current {}", mode_label(labels, &output_style));

    Menu::with_items(
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
    )
}

fn handle_menu_event(app: &AppHandle, id: &str) -> AppResult<()> {
    match id {
        "open_settings" => windows::show_main_window(app),
        "pause_voice" => {
            if let Some(state) = app.try_state::<AppState>() {
                let paused = state.toggle_paused();
                log::info!("voice input paused: {paused}");
            }
            refresh(app)?;
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
    refresh(app)?;
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
