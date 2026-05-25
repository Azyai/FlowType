use crate::{
    app::AppState,
    error::{AppError, AppResult},
    desktop::tray_i18n::tray_labels,
    desktop::windows,
};
use tauri::{
    App, AppHandle, Manager,
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
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
    let labels = app
        .try_state::<AppState>()
        .and_then(|state| {
            state
                .settings()
                .ok()
                .map(|settings| tray_labels(&settings.locale_preference))
        })
        .unwrap_or_else(|| tray_labels(&crate::settings::LocalePreference::Auto));
    let mascot_toggle_label = if mascot_is_visible(app) {
        labels.hide_mascot
    } else {
        labels.show_mascot
    };

    let open_settings = MenuItem::with_id(app, "open_settings", labels.open_settings, true, None::<&str>)?;
    let toggle_mascot = MenuItem::with_id(app, "toggle_mascot", mascot_toggle_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;

    Menu::with_items(
        app,
        &[
            &open_settings,
            &toggle_mascot,
            &quit,
        ],
    )
}

fn handle_menu_event(app: &AppHandle, id: &str) -> AppResult<()> {
    match id {
        "open_settings" => windows::show_main_window(app),
        "toggle_mascot" => {
            if mascot_is_visible(app) {
                windows::hide_mascot_windows(app)?;
            } else {
                windows::spawn_mascot_window(app)?;
            }
            refresh(app)?;
            Ok(())
        }
        "quit" => {
            app.exit(0);
            Ok(())
        }
        other => Err(AppError::Window(format!("unknown tray menu id: {other}"))),
    }
}

fn mascot_is_visible(app: &AppHandle) -> bool {
    app.get_webview_window("mascot")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false)
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
