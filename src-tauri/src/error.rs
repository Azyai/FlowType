use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("configuration file could not be read or written")]
    ConfigIo(#[from] std::io::Error),
    #[error("configuration file contains invalid JSON")]
    ConfigJson(#[from] serde_json::Error),
    #[error("database operation failed")]
    Database(#[from] rusqlite::Error),
    #[error("native window operation failed: {0}")]
    Window(String),
    #[error("startup launch operation failed: {0}")]
    Autostart(String),
    #[error("update check failed: {0}")]
    Update(String),
    #[error("ASR service configuration is missing or incomplete: {0}")]
    AsrConfigMissing(String),
    #[error("ASR service is unavailable: {0}")]
    AsrServiceUnavailable(String),
    #[error("voice input operation failed: {0}")]
    Voice(String),
    #[error("audio capture failed: {0}")]
    Audio(String),
    #[error("internal state lock failed")]
    StateLock,
}

impl From<AppError> for ErrorResponse {
    fn from(error: AppError) -> Self {
        let code = match error {
            AppError::ConfigIo(_) => "CONFIG_IO",
            AppError::ConfigJson(_) => "CONFIG_JSON",
            AppError::Database(_) => "DATABASE",
            AppError::Window(_) => "WINDOW",
            AppError::Autostart(_) => "AUTOSTART",
            AppError::Update(_) => "UPDATE",
            AppError::AsrConfigMissing(_) => "ASR_CONFIG_MISSING",
            AppError::AsrServiceUnavailable(_) => "ASR_SERVICE_UNAVAILABLE",
            AppError::Voice(_) => "VOICE_INPUT",
            AppError::Audio(_) => "AUDIO_CAPTURE",
            AppError::StateLock => "STATE_LOCK",
        };

        Self {
            code,
            message: error.to_string(),
            details: Some(format!("{error:?}")),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
pub type CommandResult<T> = Result<T, ErrorResponse>;
