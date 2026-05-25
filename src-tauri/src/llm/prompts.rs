use crate::settings::FormalScene;

use super::formal_skills;

const CLEAN_SYSTEM_PROMPT: &str = r#"You clean ASR dictation into readable text.

Rules:
- Preserve the original meaning.
- Remove filler words and hesitation where appropriate.
- Add basic punctuation if needed.
- Lightly fix spacing in mixed Chinese and English text.
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not invent facts.
- Do not summarize.
- Do not translate the original text into another language.
- Return only the cleaned text."#;

const FORMAL_SYSTEM_PROMPT_PREFIX: &str = r#"You rewrite ASR dictation into formal written text.

Global rules:
- Preserve the original meaning.
- Improve wording, punctuation, and structure.
- Keep the output language aligned with the ASR input language.
- If the ASR input is Chinese, return Chinese.
- If the ASR input is English, return English.
- If the ASR input is mixed Chinese and English, keep the mixed-language structure.
- Do not invent facts.
- Do not add markdown, titles, or explanations.
- Do not translate the original text into another language.
- Return only the final text.

Scene skill:"#;

pub fn clean_system_prompt() -> &'static str {
    CLEAN_SYSTEM_PROMPT
}

pub fn formal_system_prompt(scene: FormalScene) -> String {
    format!("{FORMAL_SYSTEM_PROMPT_PREFIX}\n{}", formal_skills::skill_prompt(scene))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_prompt_includes_scene_skill() {
        let prompt = formal_system_prompt(FormalScene::Greeting);

        assert!(prompt.contains("Scene skill:"));
        assert!(prompt.contains("Skill: Greeting rewrite"));
        assert!(prompt.contains("Keep the output language aligned with the ASR input language."));
    }
}
