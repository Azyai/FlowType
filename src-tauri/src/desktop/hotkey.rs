use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use crate::app::AppState;

lazy_static::lazy_static! {
    pub static ref IS_RECORDING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

/// Starts a global hotkey listener for hold-to-talk.
/// Only manages IS_RECORDING flag — does NOT emit status_changed events,
/// so the mascot's visual state is only controlled by double-click toggle.
pub fn start_hotkey_listener(app_handle: AppHandle) {
    let is_listening_clone = IS_RECORDING.clone();
    
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            let target_key = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(settings) = state.settings() {
                    settings.hotkey.to_lowercase()
                } else {
                    "alt".to_string()
                }
            } else {
                "alt".to_string()
            };

            let is_match = match event.event_type {
                EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                    match key {
                        Key::Alt | Key::AltGr => target_key.contains("alt"),
                        Key::ControlLeft | Key::ControlRight => target_key.contains("ctrl"),
                        Key::ShiftLeft | Key::ShiftRight => target_key.contains("shift"),
                        Key::MetaLeft | Key::MetaRight => target_key.contains("meta"),
                        Key::Space => target_key.contains("space"),
                        _ => {
                            let name = format!("{:?}", key).to_lowercase();
                            target_key.contains(&name.replace("key", ""))
                        }
                    }
                }
                _ => false,
            };

            if !is_match {
                return;
            }

            match event.event_type {
                EventType::KeyPress(_) => {
                    if !is_listening_clone.load(Ordering::SeqCst) {
                        is_listening_clone.store(true, Ordering::SeqCst);
                    }
                }
                EventType::KeyRelease(_) => {
                    if is_listening_clone.load(Ordering::SeqCst) {
                        is_listening_clone.store(false, Ordering::SeqCst);
                    }
                }
                _ => {}
            }
        };

        if let Err(error) = listen(callback) {
            log::error!("Error in rdev listener: {:?}", error);
        }
    });
}
