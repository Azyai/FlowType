use crate::{
    settings::{AppSettings, UpdateChannel},
    error::{AppError, AppResult},
};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

const MOCK_MANIFEST: &str = include_str!("../../mock/update-manifest.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus {
    Latest,
    Available,
    Failed,
    ChannelUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateCheckResult {
    pub status: UpdateStatus,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub channel: UpdateChannel,
    pub notes: Option<String>,
    pub manifest_url: String,
}

#[derive(Debug, Deserialize)]
struct UpdateManifest {
    channels: HashMap<String, ChannelManifest>,
}

#[derive(Debug, Deserialize)]
struct ChannelManifest {
    version: String,
    notes: Option<String>,
}

pub fn check_for_update(settings: &AppSettings, current_version: &str) -> AppResult<UpdateCheckResult> {
    let manifest = match read_manifest(&settings.update_manifest_url) {
        Ok(manifest) => manifest,
        Err(error) => return Ok(failed_result(settings, current_version, error.to_string())),
    };
    let channel_key = channel_key(settings.update_channel);
    let Some(channel) = manifest.channels.get(channel_key) else {
        return Ok(UpdateCheckResult {
            status: UpdateStatus::ChannelUnavailable,
            current_version: current_version.to_string(),
            latest_version: None,
            channel: settings.update_channel,
            notes: None,
            manifest_url: settings.update_manifest_url.clone(),
        });
    };

    let current = Version::parse(current_version).map_err(|error| AppError::Update(error.to_string()))?;
    let latest = match Version::parse(&channel.version) {
        Ok(version) => version,
        Err(error) => return Ok(failed_result(settings, current_version, error.to_string())),
    };
    let status = if latest > current {
        UpdateStatus::Available
    } else {
        UpdateStatus::Latest
    };

    Ok(UpdateCheckResult {
        status,
        current_version: current_version.to_string(),
        latest_version: Some(channel.version.clone()),
        channel: settings.update_channel,
        notes: channel.notes.clone(),
        manifest_url: settings.update_manifest_url.clone(),
    })
}

fn failed_result(settings: &AppSettings, current_version: &str, reason: String) -> UpdateCheckResult {
    UpdateCheckResult {
        status: UpdateStatus::Failed,
        current_version: current_version.to_string(),
        latest_version: None,
        channel: settings.update_channel,
        notes: Some(reason),
        manifest_url: settings.update_manifest_url.clone(),
    }
}

fn read_manifest(url: &str) -> AppResult<UpdateManifest> {
    let raw = if url.starts_with("mock://") {
        MOCK_MANIFEST.to_string()
    } else if Path::new(url).exists() {
        fs::read_to_string(url)?
    } else {
        return Err(AppError::Update(format!("manifest is not available: {url}")));
    };

    serde_json::from_str(&raw).map_err(|error| AppError::Update(error.to_string()))
}

fn channel_key(channel: UpdateChannel) -> &'static str {
    match channel {
        UpdateChannel::Stable => "stable",
        UpdateChannel::Beta => "beta",
        UpdateChannel::Dev => "dev",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn mock_manifest_reports_newer_stable_version() {
        let settings = AppSettings::default();

        let result = check_for_update(&settings, "0.1.0").unwrap();

        assert_eq!(result.status, UpdateStatus::Available);
        assert_eq!(result.latest_version, Some("0.1.1".to_string()));
        assert_eq!(result.channel, UpdateChannel::Stable);
    }

    #[test]
    fn equal_remote_version_reports_latest() {
        let settings = AppSettings::default();

        let result = check_for_update(&settings, "0.1.1").unwrap();

        assert_eq!(result.status, UpdateStatus::Latest);
    }

    #[test]
    fn missing_channel_reports_channel_unavailable() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let manifest_path = std::env::temp_dir().join(format!("flowtype-manifest-{unique}.json"));
        fs::write(&manifest_path, r#"{"channels":{}}"#).unwrap();
        let settings = AppSettings {
            update_manifest_url: manifest_path.display().to_string(),
            ..AppSettings::default()
        };

        let result = check_for_update(&settings, "0.1.0").unwrap();

        assert_eq!(result.status, UpdateStatus::ChannelUnavailable);
        assert_eq!(result.latest_version, None);
    }

    #[test]
    fn unavailable_manifest_reports_failed_status() {
        let settings = AppSettings {
            update_manifest_url: "F:/definitely/missing/flowtype-update.json".to_string(),
            ..AppSettings::default()
        };

        let result = check_for_update(&settings, "0.1.0").unwrap();

        assert_eq!(result.status, UpdateStatus::Failed);
        assert_eq!(result.latest_version, None);
        assert!(result.notes.unwrap().contains("manifest is not available"));
    }
}
