use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEEPSEEK_CHAT_COMPLETIONS_URL: &str = "https://api.deepseek.com/chat/completions";
const DEEPSEEK_API_KEY: &str = "sk-71b962f602854f358ea2c970ab80b436";

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
        .set("Authorization", &format!("Bearer {DEEPSEEK_API_KEY}"))
        .set("Content-Type", "application/json")
        .send_json(ureq::json!(request))
        .map_err(map_transport_error)?;

    if response.status() != 200 {
        return Err(AppError::Llm(format!(
            "DeepSeek returned HTTP {} from {}.",
            response.status(),
            DEEPSEEK_BASE_URL
        )));
    }

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

fn map_transport_error(error: ureq::Error) -> AppError {
    match error {
        ureq::Error::Status(status, response) => AppError::Llm(format!(
            "DeepSeek request failed with HTTP {status}: {}",
            response.status_text()
        )),
        ureq::Error::Transport(transport) => AppError::Llm(format!("DeepSeek transport error: {transport}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_names_match_deepseek_docs() {
        assert_eq!(DeepSeekModel::V4Flash.as_str(), "deepseek-v4-flash");
        assert_eq!(DeepSeekModel::V4Pro.as_str(), "deepseek-v4-pro");
    }
}
