use crate::settings::{AppSettings, AsrServiceMode, LocalePreference};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const PROVIDER: &str = "iflytek";
const BUILT_IN_APP_ID: &str = "6c857501";
const BUILT_IN_API_KEY: &str = "e116bd402a353297f41a7ac7b9bc2bb2";
const BUILT_IN_API_SECRET: &str = "NWU5NGQ0MDkxZDI4MmU5NDZhOGE3ZDY5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsrServiceStatusKind {
    Ready,
    MissingConfig,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsrServiceConfig {
    pub provider: &'static str,
    pub service_mode: AsrServiceMode,
    pub iflytek_app_id_masked: String,
    pub iflytek_api_key_masked: String,
    pub iflytek_api_secret_configured: bool,
    pub language: crate::settings::IflytekLanguage,
    pub mixed_language: bool,
    pub timeout_ms: u64,
    pub retry_count: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct AsrServiceCheckResult {
    pub status: AsrServiceStatusKind,
    pub provider: &'static str,
    pub service_mode: AsrServiceMode,
    pub message: String,
    pub missing_fields: Vec<&'static str>,
    pub checked_at: String,
}

pub fn service_config(settings: &AppSettings) -> AsrServiceConfig {
    let (app_id, api_key, api_secret_configured) = match settings.asr_service_mode {
        AsrServiceMode::BuiltIn => (BUILT_IN_APP_ID, BUILT_IN_API_KEY, !BUILT_IN_API_SECRET.is_empty()),
        AsrServiceMode::CustomDev => (
            settings.iflytek_app_id.as_str(),
            settings.iflytek_api_key.as_str(),
            !settings.iflytek_api_secret.trim().is_empty(),
        ),
    };

    AsrServiceConfig {
        provider: PROVIDER,
        service_mode: settings.asr_service_mode,
        iflytek_app_id_masked: mask_secret(app_id),
        iflytek_api_key_masked: mask_secret(api_key),
        iflytek_api_secret_configured: api_secret_configured,
        language: settings.iflytek_language,
        mixed_language: settings.iflytek_mixed_language,
        timeout_ms: settings.iflytek_timeout_ms,
        retry_count: settings.iflytek_retry_count,
    }
}

pub fn check_service(settings: &AppSettings) -> AsrServiceCheckResult {
    match settings.asr_service_mode {
        AsrServiceMode::BuiltIn => AsrServiceCheckResult {
            status: AsrServiceStatusKind::Ready,
            provider: PROVIDER,
            service_mode: settings.asr_service_mode,
            message: asr_message(&settings.locale_preference, AsrMessage::BuiltInReady),
            missing_fields: vec![],
            checked_at: now_string(),
        },
        AsrServiceMode::CustomDev => {
            let missing_fields = missing_custom_dev_fields(settings);
            if missing_fields.is_empty() {
                AsrServiceCheckResult {
                    status: AsrServiceStatusKind::Ready,
                    provider: PROVIDER,
                    service_mode: settings.asr_service_mode,
                    message: asr_message(&settings.locale_preference, AsrMessage::CustomDevReady),
                    missing_fields,
                    checked_at: now_string(),
                }
            } else {
                AsrServiceCheckResult {
                    status: AsrServiceStatusKind::MissingConfig,
                    provider: PROVIDER,
                    service_mode: settings.asr_service_mode,
                    message: asr_message(&settings.locale_preference, AsrMessage::CustomDevMissing),
                    missing_fields,
                    checked_at: now_string(),
                }
            }
        }
    }
}

enum AsrMessage {
    BuiltInReady,
    CustomDevReady,
    CustomDevMissing,
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
        (true, AsrMessage::BuiltInReady) => "内置科大讯飞服务已配置，可用于后续识别流程。",
        (false, AsrMessage::BuiltInReady) => {
            "Built-in iFlytek service is configured for the recognition flow."
        }
        (true, AsrMessage::CustomDevReady) => "开发期科大讯飞凭据已配置完整，可进入后续联调。",
        (false, AsrMessage::CustomDevReady) => {
            "iFlytek development credentials are complete for later integration."
        }
        (true, AsrMessage::CustomDevMissing) => "开发期科大讯飞凭据不完整。",
        (false, AsrMessage::CustomDevMissing) => "iFlytek development credentials are incomplete.",
    };

    text.to_string()
}

fn missing_custom_dev_fields(settings: &AppSettings) -> Vec<&'static str> {
    let mut missing = Vec::new();
    if settings.iflytek_app_id.trim().is_empty() {
        missing.push("app_id");
    }
    if settings.iflytek_api_key.trim().is_empty() {
        missing.push("api_key");
    }
    if settings.iflytek_api_secret.trim().is_empty() {
        missing.push("api_secret");
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
    use crate::settings::{AppSettings, AsrServiceMode};

    #[test]
    fn built_in_config_returns_only_masked_credentials() {
        let settings = AppSettings::default();

        let config = service_config(&settings);
        let debug_text = format!("{config:?}");

        assert_eq!(config.iflytek_app_id_masked, "***");
        assert_eq!(config.iflytek_api_key_masked, "e116***2bb2");
        assert!(config.iflytek_api_secret_configured);
        assert!(!debug_text.contains(BUILT_IN_APP_ID));
        assert!(!debug_text.contains(BUILT_IN_API_KEY));
        assert!(!debug_text.contains(BUILT_IN_API_SECRET));
    }

    #[test]
    fn masks_custom_dev_credentials_without_revealing_secret() {
        let mut settings = AppSettings::default();
        settings.asr_service_mode = AsrServiceMode::CustomDev;
        settings.iflytek_app_id = "appid123456".to_string();
        settings.iflytek_api_key = "apikey123456".to_string();
        settings.iflytek_api_secret = "secret123456".to_string();

        let config = service_config(&settings);

        assert_eq!(config.iflytek_app_id_masked, "appi***3456");
        assert_eq!(config.iflytek_api_key_masked, "apik***3456");
        assert!(config.iflytek_api_secret_configured);
        assert!(!format!("{config:?}").contains("secret123456"));
    }

    #[test]
    fn built_in_service_reports_ready() {
        let mut settings = AppSettings::default();
        settings.locale_preference = LocalePreference::ZhCn;

        let status = check_service(&settings);

        assert_eq!(status.status, AsrServiceStatusKind::Ready);
        assert_eq!(status.message, "内置科大讯飞服务已配置，可用于后续识别流程。");
        assert!(status.missing_fields.is_empty());
    }

    #[test]
    fn custom_dev_service_reports_missing_fields() {
        let mut settings = AppSettings::default();
        settings.asr_service_mode = AsrServiceMode::CustomDev;
        settings.locale_preference = LocalePreference::ZhCn;
        settings.iflytek_app_id = "appid".to_string();

        let status = check_service(&settings);

        assert_eq!(status.status, AsrServiceStatusKind::MissingConfig);
        assert_eq!(status.message, "开发期科大讯飞凭据不完整。");
        assert_eq!(status.missing_fields, vec!["api_key", "api_secret"]);
    }
}
