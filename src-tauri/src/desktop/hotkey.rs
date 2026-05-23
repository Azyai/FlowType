use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, Emitter};

pub fn start_hotkey_listener(app_handle: AppHandle) {
    let is_listening = Arc::new(AtomicBool::new(false));
    
    // We should parse the configured hotkey from settings, but for MVP we hardcode or use a global config.
    // For hold-to-talk, let's use `Alt` as a demo, or map from actual user settings.
    let is_listening_clone = is_listening.clone();
    
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    // Check if it's the target hotkey (e.g. Alt or Custom)
                    if key == Key::Alt {
                        if !is_listening_clone.load(Ordering::SeqCst) {
                            is_listening_clone.store(true, Ordering::SeqCst);
                            let _ = app_handle.emit("status_changed", "Listening");
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    if key == Key::Alt {
                        if is_listening_clone.load(Ordering::SeqCst) {
                            is_listening_clone.store(false, Ordering::SeqCst);
                            // Transition to processing
                            let _ = app_handle.emit("status_changed", "Processing");
                            
                            // Simulate processing to inject/idle transition for now
                            let app_handle_clone = app_handle.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(Duration::from_secs(1));
                                let _ = app_handle_clone.emit("status_changed", "Injecting");
                                std::thread::sleep(Duration::from_millis(500));
                                let _ = app_handle_clone.emit("status_changed", "Idle");
                            });
                        }
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
