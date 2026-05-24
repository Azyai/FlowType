use crate::settings::{AppSettings, LocalePreference, RtasrLanguage};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const PROVIDER: &str = "xfyun_rtasr";
const BUILTIN_RTASR_APP_ID: &str = "3a5a99a1";
const BUILTIN_RTASR_API_KEY: &str = "64cc6bcce2383a37c3a6b61006a13ade";

#[derive(Debug, Clone)]
pub(crate) struct RtasrCredentials {
    pub app_id: String,
    pub api_key: String,
}

pub(crate) fn credentials_for(_settings: &AppSettings) -> Option<RtasrCredentials> {
    Some(RtasrCredentials {
        app_id: BUILTIN_RTASR_APP_ID.to_string(),
        api_key: BUILTIN_RTASR_API_KEY.to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsrServiceStatusKind {
    Ready,
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
        rtasr_app_id_masked: mask_secret(BUILTIN_RTASR_APP_ID),
        rtasr_api_key_masked: mask_secret(BUILTIN_RTASR_API_KEY),
        language: settings.rtasr_language,
        timeout_ms: settings.rtasr_timeout_ms,
    }
}

pub fn check_service(settings: &AppSettings) -> AsrServiceCheckResult {
    AsrServiceCheckResult {
        status: AsrServiceStatusKind::Ready,
        provider: PROVIDER,
        message: asr_message(&settings.locale_preference, AsrMessage::Ready),
        missing_fields: vec![],
        checked_at: now_string(),
    }
}

enum AsrMessage {
    Ready,
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
    };

    text.to_string()
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
        let settings = AppSettings::default();

        let config = service_config(&settings);
        let debug_text = format!("{config:?}");

        assert_eq!(config.rtasr_app_id_masked, "***");
        assert_eq!(config.rtasr_api_key_masked, "64cc***3ade");
        assert!(!debug_text.contains(BUILTIN_RTASR_APP_ID));
        assert!(!debug_text.contains(BUILTIN_RTASR_API_KEY));
    }

    #[test]
    fn credentials_always_use_builtin_values() {
        let settings = AppSettings::default();

        let credentials = credentials_for(&settings).unwrap();

        assert_eq!(credentials.app_id, BUILTIN_RTASR_APP_ID);
        assert_eq!(credentials.api_key, BUILTIN_RTASR_API_KEY);
    }

    #[test]
    fn service_reports_ready_with_builtin_credentials() {
        let mut settings = AppSettings::default();
        settings.locale_preference = LocalePreference::ZhCn;

        let status = check_service(&settings);

        assert_eq!(status.status, AsrServiceStatusKind::Ready);
        assert_eq!(status.message, "实时转写 RTASR 凭据已配置完整。");
        assert!(status.missing_fields.is_empty());
    }
}
