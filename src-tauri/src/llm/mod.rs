pub mod deepseek;
pub mod formal_skills;
pub mod prompts;

use crate::{
    error::AppResult,
    settings::{FormalScene, OutputStyle},
};
use deepseek::{rewrite_text as deepseek_rewrite_text, DeepSeekModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteResult {
    pub text: String,
    pub provider: &'static str,
    pub model: Option<DeepSeekModel>,
    pub scene: Option<FormalScene>,
    pub latency_ms: u128,
    pub fallback_used: bool,
}

pub fn rewrite_text(
    raw_text: &str,
    output_style: OutputStyle,
    formal_scene: FormalScene,
) -> AppResult<RewriteResult> {
    match output_style {
        OutputStyle::Raw => Ok(RewriteResult {
            text: raw_text.trim().to_string(),
            provider: "native",
            model: None,
            scene: None,
            latency_ms: 0,
            fallback_used: false,
        }),
        OutputStyle::Clean => {
            let response =
                deepseek_rewrite_text(DeepSeekModel::V4Flash, prompts::clean_system_prompt(), raw_text)?;
            Ok(RewriteResult {
                text: response.text,
                provider: "deepseek",
                model: Some(response.model),
                scene: None,
                latency_ms: response.latency_ms,
                fallback_used: false,
            })
        }
        OutputStyle::Formal => {
            let response = deepseek_rewrite_text(
                DeepSeekModel::V4Pro,
                &prompts::formal_system_prompt(formal_scene),
                raw_text,
            )?;
            Ok(RewriteResult {
                text: response.text,
                provider: "deepseek",
                model: Some(response.model),
                scene: Some(formal_scene),
                latency_ms: response.latency_ms,
                fallback_used: false,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_mode_returns_trimmed_text_without_model() {
        let result = rewrite_text("  hello world  ", OutputStyle::Raw, FormalScene::General).unwrap();

        assert_eq!(result.text, "hello world");
        assert_eq!(result.provider, "native");
        assert_eq!(result.model, None);
        assert_eq!(result.scene, None);
    }
}
