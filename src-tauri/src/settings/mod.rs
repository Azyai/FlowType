use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    HoldToTalk,
    Toggle,
}

fn default_hotkey() -> String {
    "Ctrl+Alt+V".to_string()
}

fn default_input_mode() -> InputMode {
    InputMode::HoldToTalk
}

fn default_toggle_hotkey() -> String {
    "Ctrl+Alt+M".to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RtasrLanguage {
    ZhCn,
    EnUs,
    ZhEn,
}

fn default_rtasr_language() -> RtasrLanguage {
    RtasrLanguage::ZhEn
}

fn default_rtasr_timeout_ms() -> u64 {
    10_000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStyle {
    Raw,
    Clean,
    Formal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardRestore {
    Always,
    Delayed,
    Never,
    TextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatingWindowPosition {
    BottomRight,
    CursorNearby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Stable,
    Beta,
    Dev,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalePreference {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
}

fn default_locale_preference() -> LocalePreference {
    LocalePreference::Auto
}

fn default_history_retention_days() -> u16 {
    14
}

fn default_true() -> bool {
    true
}

fn default_min_recording_ms() -> u64 {
    500
}

fn default_max_recording_ms() -> u64 {
    60_000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_input_mode")]
    pub input_mode: InputMode,
    #[serde(default = "default_toggle_hotkey")]
    pub toggle_hotkey: String,
    #[serde(default, alias = "iflytek_app_id")]
    pub rtasr_app_id: String,
    #[serde(default, alias = "iflytek_api_key")]
    pub rtasr_api_key: String,
    #[serde(default = "default_rtasr_language", alias = "iflytek_language")]
    pub rtasr_language: RtasrLanguage,
    #[serde(default = "default_rtasr_timeout_ms", alias = "iflytek_timeout_ms")]
    pub rtasr_timeout_ms: u64,
    pub output_style: OutputStyle,
    pub clipboard_restore: ClipboardRestore,
    pub floating_window_position: FloatingWindowPosition,
    #[serde(default = "default_true")]
    pub show_floating_window: bool,
    #[serde(default = "default_true")]
    pub floating_window_always_on_top: bool,
    #[serde(default = "default_true")]
    pub floating_window_animation_enabled: bool,
    pub save_history: bool,
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u16,
    #[serde(default)]
    pub vad_enabled: bool,
    #[serde(default)]
    pub hotwords_enabled: bool,
    #[serde(default = "default_min_recording_ms")]
    pub min_recording_ms: u64,
    #[serde(default = "default_max_recording_ms")]
    pub max_recording_ms: u64,
    pub auto_start: bool,
    pub update_channel: UpdateChannel,
    pub update_manifest_url: String,
    pub auto_check_update: bool,
    #[serde(default = "default_locale_preference")]
    pub locale_preference: LocalePreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            input_mode: InputMode::HoldToTalk,
            toggle_hotkey: default_toggle_hotkey(),
            rtasr_app_id: String::new(),
            rtasr_api_key: String::new(),
            rtasr_language: RtasrLanguage::ZhEn,
            rtasr_timeout_ms: 10_000,
            output_style: OutputStyle::Raw,
            clipboard_restore: ClipboardRestore::Never,
            floating_window_position: FloatingWindowPosition::BottomRight,
            show_floating_window: true,
            floating_window_always_on_top: true,
            floating_window_animation_enabled: true,
            save_history: true,
            history_retention_days: 14,
            vad_enabled: false,
            hotwords_enabled: false,
            min_recording_ms: 500,
            max_recording_ms: 60_000,
            auto_start: false,
            update_channel: UpdateChannel::Stable,
            update_manifest_url: "mock://updates/stable.json".to_string(),
            auto_check_update: false,
            locale_preference: LocalePreference::Auto,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> AppResult<AppSettings> {
        if !self.path.exists() {
            let settings = AppSettings::default();
            self.save(&settings)?;
            return Ok(settings);
        }

        let text = fs::read_to_string(&self.path)?;
        let has_toggle_hotkey = text.contains("\"toggle_hotkey\"");
        match serde_json::from_str::<AppSettings>(&text) {
            Ok(mut settings) => {
                settings.clipboard_restore = ClipboardRestore::Never;
                normalize_hotkeys(&mut settings, has_toggle_hotkey);
                Ok(settings)
            }
            Err(_) => {
                self.move_corrupt_file()?;
                let settings = AppSettings::default();
                self.save(&settings)?;
                Ok(settings)
            }
        }
    }

    pub fn save(&self, settings: &AppSettings) -> AppResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        if self.path.exists() {
            fs::copy(&self.path, self.path.with_extension("json.bak"))?;
        }

        let temp_path = self.path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(settings)?;
        fs::write(&temp_path, text)?;

        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        fs::rename(temp_path, &self.path)?;
        Ok(())
    }

    fn move_corrupt_file(&self) -> AppResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or_default();
        let corrupt_path = self.path.with_extension(format!("corrupt.{timestamp}.json"));
        fs::rename(&self.path, corrupt_path)?;
        Ok(())
    }
}

fn normalize_hotkeys(settings: &mut AppSettings, has_toggle_hotkey: bool) {
    let migrated_legacy_toggle = !has_toggle_hotkey && matches!(settings.input_mode, InputMode::Toggle);

    if settings.hotkey.trim().is_empty() {
        settings.hotkey = default_hotkey();
    }

    if settings.toggle_hotkey.trim().is_empty() || migrated_legacy_toggle {
        settings.toggle_hotkey = if migrated_legacy_toggle {
            settings.hotkey.clone()
        } else {
            default_toggle_hotkey()
        };
    }

    if same_hotkey(&settings.hotkey, &settings.toggle_hotkey) {
        if migrated_legacy_toggle {
            settings.hotkey = if same_hotkey(&default_hotkey(), &settings.toggle_hotkey) {
                default_toggle_hotkey()
            } else {
                default_hotkey()
            };
        } else {
            settings.toggle_hotkey = if same_hotkey(&default_toggle_hotkey(), &settings.hotkey) {
                default_hotkey()
            } else {
                default_toggle_hotkey()
            };
        }
    }
}

fn same_hotkey(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    !left.is_empty() && left.eq_ignore_ascii_case(right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("flowtype-{name}-{unique}")).join("settings.json")
    }

    #[test]
    fn default_settings_match_phase_zero_defaults() {
        let settings = AppSettings::default();

        assert_eq!(settings.hotkey, "Ctrl+Alt+V");
        assert_eq!(settings.input_mode, InputMode::HoldToTalk);
        assert_eq!(settings.toggle_hotkey, "Ctrl+Alt+M");
        assert_eq!(settings.rtasr_language, RtasrLanguage::ZhEn);
        assert_eq!(settings.rtasr_timeout_ms, 10_000);
        assert_eq!(settings.output_style, OutputStyle::Raw);
        assert_eq!(settings.clipboard_restore, ClipboardRestore::Never);
        assert_eq!(settings.floating_window_position, FloatingWindowPosition::BottomRight);
        assert!(settings.show_floating_window);
        assert!(settings.floating_window_always_on_top);
        assert!(settings.floating_window_animation_enabled);
        assert!(settings.save_history);
        assert_eq!(settings.history_retention_days, 14);
        assert!(!settings.vad_enabled);
        assert!(!settings.hotwords_enabled);
        assert_eq!(settings.min_recording_ms, 500);
        assert_eq!(settings.max_recording_ms, 60_000);
        assert!(!settings.auto_start);
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
        assert_eq!(settings.update_manifest_url, "mock://updates/stable.json");
        assert!(!settings.auto_check_update);
        assert_eq!(settings.locale_preference, LocalePreference::Auto);
    }

    #[test]
    fn old_settings_load_with_rtasr_aliases() {
        let path = test_path("legacy");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{
              "hotkey": "Alt",
              "input_mode": "hold_to_talk",
              "iflytek_app_id": "legacy-app-id",
              "iflytek_api_key": "legacy-api-key",
              "iflytek_language": "zh_en",
              "iflytek_timeout_ms": 8000,
              "output_style": "clean",
              "clipboard_restore": "always",
              "floating_window_position": "bottom_right",
              "save_history": true,
              "auto_start": false,
              "update_channel": "stable",
              "update_manifest_url": "mock://updates/stable.json",
              "auto_check_update": false
            }"#,
        )
        .unwrap();
        let store = ConfigStore::new(&path);

        let loaded = store.load().unwrap();

        assert_eq!(loaded.rtasr_app_id, "legacy-app-id");
        assert_eq!(loaded.rtasr_api_key, "legacy-api-key");
        assert_eq!(loaded.rtasr_language, RtasrLanguage::ZhEn);
        assert_eq!(loaded.rtasr_timeout_ms, 8_000);
        assert_eq!(loaded.output_style, OutputStyle::Clean);
        assert_eq!(loaded.clipboard_restore, ClipboardRestore::Never);
        assert_eq!(loaded.toggle_hotkey, "Ctrl+Alt+M");
        assert_eq!(loaded.history_retention_days, 14);
        assert_eq!(loaded.min_recording_ms, 500);
        assert_eq!(loaded.max_recording_ms, 60_000);
    }

    #[test]
    fn load_creates_default_settings_file_when_missing() {
        let path = test_path("missing");
        let store = ConfigStore::new(&path);

        let settings = store.load().unwrap();

        assert_eq!(settings, AppSettings::default());
        assert!(path.exists());
    }

    #[test]
    fn save_and_load_round_trip_user_settings() {
        let path = test_path("roundtrip");
        let store = ConfigStore::new(&path);
        let mut settings = AppSettings::default();
        settings.hotkey = "Ctrl+Space".to_string();
        settings.toggle_hotkey = "Ctrl+Alt+M".to_string();
        settings.auto_start = true;
        settings.update_channel = UpdateChannel::Beta;

        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();

        assert_eq!(loaded, settings);
    }

    #[test]
    fn corrupt_config_is_moved_and_replaced_with_defaults() {
        let path = test_path("corrupt");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{ definitely-not-json").unwrap();
        let store = ConfigStore::new(&path);

        let loaded = store.load().unwrap();
        let corrupt_files = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("corrupt"))
            .count();

        assert_eq!(loaded, AppSettings::default());
        assert_eq!(corrupt_files, 1);
        assert!(path.exists());
    }

    #[test]
    fn old_toggle_only_settings_migrate_to_dual_hotkeys() {
        let path = test_path("legacy-toggle-hotkey");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{
              "hotkey": "Alt",
              "input_mode": "toggle",
              "output_style": "raw",
              "clipboard_restore": "never",
              "floating_window_position": "bottom_right",
              "save_history": true,
              "auto_start": false,
              "update_channel": "stable",
              "update_manifest_url": "mock://updates/stable.json",
              "auto_check_update": false
            }"#,
        )
        .unwrap();
        let store = ConfigStore::new(&path);

        let loaded = store.load().unwrap();

        assert_eq!(loaded.toggle_hotkey, "Alt");
        assert_eq!(loaded.hotkey, "Ctrl+Alt+V");
    }
}
