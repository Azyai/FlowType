use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputMode {
    HoldToTalk,
    Toggle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AsrMode {
    LocalFirst,
    CloudFirst,
    CloudOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStyle {
    Raw,
    Clean,
    Formal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardRestore {
    Always,
    Delayed,
    Never,
    TextOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub hotkey: String,
    pub input_mode: InputMode,
    pub asr_mode: AsrMode,
    pub default_model: String,
    pub output_style: OutputStyle,
    pub clipboard_restore: ClipboardRestore,
    pub floating_window_position: FloatingWindowPosition,
    pub save_history: bool,
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
            hotkey: "Alt".to_string(),
            input_mode: InputMode::HoldToTalk,
            asr_mode: AsrMode::LocalFirst,
            default_model: "whisper-small-q8".to_string(),
            output_style: OutputStyle::Clean,
            clipboard_restore: ClipboardRestore::Always,
            floating_window_position: FloatingWindowPosition::BottomRight,
            save_history: true,
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
        match serde_json::from_str::<AppSettings>(&text) {
            Ok(settings) => Ok(settings),
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

        assert_eq!(settings.hotkey, "Alt");
        assert_eq!(settings.input_mode, InputMode::HoldToTalk);
        assert_eq!(settings.asr_mode, AsrMode::LocalFirst);
        assert_eq!(settings.default_model, "whisper-small-q8");
        assert_eq!(settings.output_style, OutputStyle::Clean);
        assert_eq!(settings.clipboard_restore, ClipboardRestore::Always);
        assert_eq!(settings.floating_window_position, FloatingWindowPosition::BottomRight);
        assert!(settings.save_history);
        assert!(!settings.auto_start);
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
        assert_eq!(settings.update_manifest_url, "mock://updates/stable.json");
        assert!(!settings.auto_check_update);
        assert_eq!(settings.locale_preference, LocalePreference::Auto);
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
}
