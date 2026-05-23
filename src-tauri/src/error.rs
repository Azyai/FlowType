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
    #[error("native window operation failed")]
    Window(String),
    #[error("startup launch operation failed")]
    Autostart(String),
    #[error("update check failed")]
    Update(String),
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
