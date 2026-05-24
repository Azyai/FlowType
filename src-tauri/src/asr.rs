use crate::settings::{AppSettings, LocalePreference, RtasrLanguage};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const PROVIDER: &str = "xfyun_rtasr";

#[derive(Debug, Clone)]
pub(crate) struct RtasrCredentials {
    pub app_id: String,
    pub api_key: String,
}

pub(crate) fn credentials_for(settings: &AppSettings) -> Option<RtasrCredentials> {
    if settings.rtasr_app_id.trim().is_empty() || settings.rtasr_api_key.trim().is_empty() {
        None
    } else {
        Some(RtasrCredentials {
            app_id: settings.rtasr_app_id.trim().to_string(),
            api_key: settings.rtasr_api_key.trim().to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsrServiceStatusKind {
    Ready,
    MissingConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsrServiceConfig {
    pub provider: &'static str,
    pub rtasr_app_id_masked: String,
    pub rtasr_api_key_masked: String,
    pub language: RtasrLanguage,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsrServiceCheckResult {
    pub status: AsrServiceStatusKind,
    pub provider: &'static str,
    pub message: String,
    pub missing_fields: Vec<&'static str>,
    pub checked_at: String,
}

pub fn service_config(settings: &AppSettings) -> AsrServiceConfig {
    AsrServiceConfig {
        provider: PROVIDER,
        rtasr_app_id_masked: mask_secret(&settings.rtasr_app_id),
        rtasr_api_key_masked: mask_secret(&settings.rtasr_api_key),
        language: settings.rtasr_language,
        timeout_ms: settings.rtasr_timeout_ms,
    }
}

pub fn check_service(settings: &AppSettings) -> AsrServiceCheckResult {
    let missing_fields = missing_fields(settings);
    if missing_fields.is_empty() {
        AsrServiceCheckResult {
            status: AsrServiceStatusKind::Ready,
            provider: PROVIDER,
            message: asr_message(&settings.locale_preference, AsrMessage::Ready),
            missing_fields: vec![],
            checked_at: now_string(),
        }
    } else {
        AsrServiceCheckResult {
            status: AsrServiceStatusKind::MissingConfig,
            provider: PROVIDER,
            message: asr_message(&settings.locale_preference, AsrMessage::MissingConfig),
            missing_fields,
            checked_at: now_string(),
        }
    }
}

enum AsrMessage {
    Ready,
    MissingConfig,
}

fn asr_message(preference: &LocalePreference, message: AsrMessage) -> String {
    let use_zh = match preference {
        LocalePreference::ZhCn => true,
        LocalePreference::EnUs => false,
        LocalePreference::Auto => sys_locale::get_locale()
            .unwrap_or_default()
            .to_lowercase()
            .starts_with("zh"),
    };

    let text = match (use_zh, message) {
        (true, AsrMessage::Ready) => "实时转写 RTASR 凭据已配置完整。",
        (false, AsrMessage::Ready) => "RTASR credentials are configured.",
        (true, AsrMessage::MissingConfig) => "实时转写 RTASR 凭据不完整。",
        (false, AsrMessage::MissingConfig) => "RTASR credentials are incomplete.",
    };

    text.to_string()
}

fn missing_fields(settings: &AppSettings) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if settings.rtasr_app_id.trim().is_empty() {
        missing.push("app_id");
    }
    if settings.rtasr_api_key.trim().is_empty() {
        missing.push("api_key");
    }
    missing
}

fn mask_secret(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= 8 {
        return "***".to_string();
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{prefix}***{suffix}")
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppSettings;

    #[test]
    fn config_returns_only_masked_credentials() {
        let mut settings = AppSettings::default();
        settings.rtasr_app_id = "appid123456".to_string();
        settings.rtasr_api_key = "apikey123456".to_string();

        let config = service_config(&settings);
        let debug_text = format!("{config:?}");

        assert_eq!(config.rtasr_app_id_masked, "appi***3456");
        assert_eq!(config.rtasr_api_key_masked, "apik***3456");
        assert!(!debug_text.contains("appid123456"));
        assert!(!debug_text.contains("apikey123456"));
    }

    #[test]
    fn credentials_are_available_when_required_fields_exist() {
        let mut settings = AppSettings::default();
        settings.rtasr_app_id = "appid123456".to_string();
        settings.rtasr_api_key = "apikey123456".to_string();

        let credentials = credentials_for(&settings).unwrap();

        assert_eq!(credentials.app_id, "appid123456");
        assert_eq!(credentials.api_key, "apikey123456");
    }

    #[test]
    fn service_reports_ready_when_credentials_exist() {
        let mut settings = AppSettings::default();
        settings.locale_preference = LocalePreference::ZhCn;
        settings.rtasr_app_id = "appid".to_string();
        settings.rtasr_api_key = "apikey".to_string();

        let status = check_service(&settings);

        assert_eq!(status.status, AsrServiceStatusKind::Ready);
        assert_eq!(status.message, "实时转写 RTASR 凭据已配置完整。");
        assert!(status.missing_fields.is_empty());
    }

    #[test]
    fn service_reports_missing_fields() {
        let mut settings = AppSettings::default();
        settings.locale_preference = LocalePreference::ZhCn;
        settings.rtasr_app_id = "appid".to_string();

        let status = check_service(&settings);

        assert_eq!(status.status, AsrServiceStatusKind::MissingConfig);
        assert_eq!(status.message, "实时转写 RTASR 凭据不完整。");
        assert_eq!(status.missing_fields, vec!["api_key"]);
    }
}
