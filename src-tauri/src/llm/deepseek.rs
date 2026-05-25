use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Instant;

const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEEPSEEK_CHAT_COMPLETIONS_URL: &str = "https://api.deepseek.com/chat/completions";
const DEEPSEEK_API_KEY_ENV: &str = "DEEP_SEEK_API_KEY";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeepSeekModel {
    V4Flash,
    V4Pro,
}

impl DeepSeekModel {
    pub fn as_str(self) -> &'static str {
        match self {
            DeepSeekModel::V4Flash => "deepseek-v4-flash",
            DeepSeekModel::V4Pro => "deepseek-v4-pro",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeepSeekResponse {
    pub text: String,
    pub model: DeepSeekModel,
    pub latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct ChatCompletionsRequest<'a> {
    model: &'a str,
    messages: [ChatMessage<'a>; 2],
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
struct AssistantMessage {
    content: String,
}

pub fn rewrite_text(model: DeepSeekModel, system_prompt: &str, user_text: &str) -> AppResult<DeepSeekResponse> {
    let api_key = load_api_key()?;
    let started_at = Instant::now();
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(30))
        .timeout_write(std::time::Duration::from_secs(30))
        .build();

    let request = ChatCompletionsRequest {
        model: model.as_str(),
        messages: [
            ChatMessage {
                role: "system",
                content: system_prompt,
            },
            ChatMessage {
                role: "user",
                content: user_text,
            },
        ],
        stream: false,
    };

    let response = agent
        .post(DEEPSEEK_CHAT_COMPLETIONS_URL)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Content-Type", "application/json")
        .send_json(ureq::json!(request))
        .map_err(map_transport_error)?;

    let payload: ChatCompletionsResponse = response
        .into_json()
        .map_err(|error| AppError::Llm(format!("DeepSeek returned invalid JSON: {error}")))?;

    let text = payload
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content.trim().to_string())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| AppError::Llm("DeepSeek returned an empty response.".to_string()))?;

    Ok(DeepSeekResponse {
        text,
        model,
        latency_ms: started_at.elapsed().as_millis(),
    })
}

fn load_api_key() -> AppResult<String> {
    let api_key = env::var(DEEPSEEK_API_KEY_ENV).map_err(|_| {
        AppError::Llm(format!(
            "DeepSeek API key is missing. Please set the local environment variable {DEEPSEEK_API_KEY_ENV}."
        ))
    })?;

    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(AppError::Llm(format!(
            "DeepSeek API key is empty. Please set a non-empty value in {DEEPSEEK_API_KEY_ENV}."
        )));
    }

    Ok(trimmed.to_string())
}

fn map_transport_error(error: ureq::Error) -> AppError {
    match error {
        ureq::Error::Status(status, response) => {
            let reason = response.status_text().trim().to_string();
            match status {
                401 => AppError::Llm(format!(
                    "DeepSeek authentication failed with HTTP 401 from {DEEPSEEK_BASE_URL}: {reason}"
                )),
                429 => AppError::Llm(format!(
                    "DeepSeek rate limit reached with HTTP 429 from {DEEPSEEK_BASE_URL}: {reason}"
                )),
                _ => AppError::Llm(format!(
                    "DeepSeek request failed with HTTP {status} from {DEEPSEEK_BASE_URL}: {reason}"
                )),
            }
        }
        ureq::Error::Transport(transport) => {
            let message = transport.to_string();
            if message.to_ascii_lowercase().contains("timed out") || message.to_ascii_lowercase().contains("timeout")
            {
                AppError::Llm(format!("DeepSeek request timed out while contacting {DEEPSEEK_BASE_URL}."))
            } else {
                AppError::Llm(format!("DeepSeek transport error while contacting {DEEPSEEK_BASE_URL}: {message}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn model_names_match_deepseek_docs() {
        assert_eq!(DeepSeekModel::V4Flash.as_str(), "deepseek-v4-flash");
        assert_eq!(DeepSeekModel::V4Pro.as_str(), "deepseek-v4-pro");
    }

    #[test]
    fn load_api_key_reads_from_environment() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::set_var(DEEPSEEK_API_KEY_ENV, " test-key "); }

        let api_key = load_api_key().unwrap();

        assert_eq!(api_key, "test-key");
        unsafe { env::remove_var(DEEPSEEK_API_KEY_ENV); }
    }

    #[test]
    fn load_api_key_returns_error_when_missing() {
        let _guard = env_lock().lock().unwrap();
        unsafe { env::remove_var(DEEPSEEK_API_KEY_ENV); }

        let error = load_api_key().unwrap_err();

        match error {
            AppError::Llm(message) => assert!(message.contains(DEEPSEEK_API_KEY_ENV)),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
