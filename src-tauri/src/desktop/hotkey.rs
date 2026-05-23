use crate::{
    app::AppState,
    settings::InputMode,
    voice::state::{VoiceStatus, VoiceTrigger},
};
use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyCombo {
    keys: HashSet<HotkeyKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyKey {
    Alt,
    Ctrl,
    Shift,
    Meta,
    Space,
    Character(char),
}

impl HotkeyCombo {
    pub fn parse(value: &str) -> Self {
        let keys = value
            .split('+')
            .filter_map(|part| match part.trim().to_lowercase().as_str() {
                "alt" | "option" => Some(HotkeyKey::Alt),
                "ctrl" | "control" => Some(HotkeyKey::Ctrl),
                "shift" => Some(HotkeyKey::Shift),
                "meta" | "cmd" | "command" | "win" => Some(HotkeyKey::Meta),
                "space" => Some(HotkeyKey::Space),
                text if text.len() == 1 => text.chars().next().map(|ch| HotkeyKey::Character(ch.to_ascii_uppercase())),
                _ => None,
            })
            .collect();
        Self { keys }
    }

    pub fn matches(&self, pressed: &HashSet<HotkeyKey>) -> bool {
        !self.keys.is_empty() && self.keys.iter().all(|key| pressed.contains(key))
    }

    pub fn contains(&self, key: HotkeyKey) -> bool {
        self.keys.contains(&key)
    }
}

pub fn start_hotkey_listener(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let mut pressed = HashSet::new();
        let mut hold_active = false;
        let mut toggle_latch = false;

        let callback = move |event: Event| {
            let Some(key) = event_key(&event.event_type) else {
                return;
            };
            let Some(state) = app_handle.try_state::<AppState>() else {
                return;
            };
            let settings = match state.settings() {
                Ok(settings) => settings,
                Err(error) => {
                    log::warn!("failed to read hotkey settings: {error}");
                    return;
                }
            };
            let combo = HotkeyCombo::parse(&settings.hotkey);

            match event.event_type {
                EventType::KeyPress(_) => {
                    pressed.insert(key);
                    let is_match = combo.matches(&pressed);
                    match settings.input_mode {
                        InputMode::HoldToTalk if is_match && !hold_active => {
                            hold_active = true;
                            let _ = state.start_voice_input(&app_handle, &settings, VoiceTrigger::Hotkey);
                        }
                        InputMode::Toggle if is_match && !toggle_latch => {
                            toggle_latch = true;
                            match state.voice_status() {
                                Ok(VoiceStatus::Listening) => {
                                    let _ = state.stop_voice_input(app_handle.clone(), settings.clone(), VoiceTrigger::Hotkey);
                                }
                                Ok(_) => {
                                    let _ = state.start_voice_input(&app_handle, &settings, VoiceTrigger::Hotkey);
                                }
                                Err(error) => log::warn!("failed to read voice status: {error}"),
                            }
                        }
                        _ => {}
                    }
                }
                EventType::KeyRelease(_) => {
                    pressed.remove(&key);
                    if matches!(settings.input_mode, InputMode::HoldToTalk) && hold_active && combo.contains(key) {
                        hold_active = false;
                        let _ = state.stop_voice_input(app_handle.clone(), settings.clone(), VoiceTrigger::Hotkey);
                    }
                    if matches!(settings.input_mode, InputMode::Toggle) && !combo.matches(&pressed) {
                        toggle_latch = false;
                    }
                }
                _ => {}
            }
        };

        if let Err(error) = listen(callback) {
            log::error!("global hotkey listener failed: {error:?}");
        }
    });
}

fn event_key(event_type: &EventType) -> Option<HotkeyKey> {
    match event_type {
        EventType::KeyPress(key) | EventType::KeyRelease(key) => key_to_hotkey(*key),
        _ => None,
    }
}

fn key_to_hotkey(key: Key) -> Option<HotkeyKey> {
    match key {
        Key::Alt | Key::AltGr => Some(HotkeyKey::Alt),
        Key::ControlLeft | Key::ControlRight => Some(HotkeyKey::Ctrl),
        Key::ShiftLeft | Key::ShiftRight => Some(HotkeyKey::Shift),
        Key::MetaLeft | Key::MetaRight => Some(HotkeyKey::Meta),
        Key::Space => Some(HotkeyKey::Space),
        Key::KeyA => Some(HotkeyKey::Character('A')),
        Key::KeyB => Some(HotkeyKey::Character('B')),
        Key::KeyC => Some(HotkeyKey::Character('C')),
        Key::KeyD => Some(HotkeyKey::Character('D')),
        Key::KeyE => Some(HotkeyKey::Character('E')),
        Key::KeyF => Some(HotkeyKey::Character('F')),
        Key::KeyG => Some(HotkeyKey::Character('G')),
        Key::KeyH => Some(HotkeyKey::Character('H')),
        Key::KeyI => Some(HotkeyKey::Character('I')),
        Key::KeyJ => Some(HotkeyKey::Character('J')),
        Key::KeyK => Some(HotkeyKey::Character('K')),
        Key::KeyL => Some(HotkeyKey::Character('L')),
        Key::KeyM => Some(HotkeyKey::Character('M')),
        Key::KeyN => Some(HotkeyKey::Character('N')),
        Key::KeyO => Some(HotkeyKey::Character('O')),
        Key::KeyP => Some(HotkeyKey::Character('P')),
        Key::KeyQ => Some(HotkeyKey::Character('Q')),
        Key::KeyR => Some(HotkeyKey::Character('R')),
        Key::KeyS => Some(HotkeyKey::Character('S')),
        Key::KeyT => Some(HotkeyKey::Character('T')),
        Key::KeyU => Some(HotkeyKey::Character('U')),
        Key::KeyV => Some(HotkeyKey::Character('V')),
        Key::KeyW => Some(HotkeyKey::Character('W')),
        Key::KeyX => Some(HotkeyKey::Character('X')),
        Key::KeyY => Some(HotkeyKey::Character('Y')),
        Key::KeyZ => Some(HotkeyKey::Character('Z')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_alt_and_ctrl_space_hotkeys() {
        let alt = HotkeyCombo::parse("Alt");
        let ctrl_space = HotkeyCombo::parse("Ctrl+Space");
        let mut pressed = HashSet::new();
        pressed.insert(HotkeyKey::Ctrl);
        pressed.insert(HotkeyKey::Space);

        assert!(alt.contains(HotkeyKey::Alt));
        assert!(ctrl_space.matches(&pressed));
    }

    #[test]
    fn parses_command_shift_character_hotkey() {
        let combo = HotkeyCombo::parse("Command+Shift+V");
        let mut pressed = HashSet::new();
        pressed.insert(HotkeyKey::Meta);
        pressed.insert(HotkeyKey::Shift);
        pressed.insert(HotkeyKey::Character('V'));

        assert!(combo.matches(&pressed));
    }
}
