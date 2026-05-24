use crate::settings::OutputStyle;

const LEADING_FILLERS: &[&str] = &[
    "嗯，",
    "嗯,",
    "嗯",
    "呃，",
    "呃,",
    "呃",
    "额，",
    "额,",
    "额",
    "啊，",
    "啊,",
    "啊",
    "um, ",
    "um ",
    "uh, ",
    "uh ",
];

pub fn transform_output_text(raw_text: &str, output_style: &OutputStyle) -> String {
    let normalized = normalize_line_endings(raw_text);
    match output_style {
        OutputStyle::Raw => normalized.trim().to_string(),
        OutputStyle::Clean => clean_text(&normalized),
        OutputStyle::Formal => formalize_text(&normalized),
    }
}

fn clean_text(text: &str) -> String {
    let lines = text
        .lines()
        .map(|line| collapse_inline_whitespace(line.trim()))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let cleaned = normalize_punctuation_spacing(&lines.join("\n"));
    let cleaned = strip_leading_fillers(&cleaned);
    collapse_duplicate_punctuation(&cleaned)
}

fn formalize_text(text: &str) -> String {
    let mut formal = clean_text(text);
    if formal.is_empty() {
        return formal;
    }

    if contains_cjk(&formal) {
        formal = standardize_cjk_punctuation(&formal);
    } else {
        formal = capitalize_first_ascii_letter(&formal);
    }

    if !ends_with_terminal_punctuation(&formal) {
        formal.push(if contains_cjk(&formal) { '。' } else { '.' });
    }

    collapse_duplicate_punctuation(&formal)
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn collapse_inline_whitespace(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut in_whitespace = false;

    for character in text.chars() {
        if character.is_whitespace() {
            if !in_whitespace {
                output.push(' ');
                in_whitespace = true;
            }
        } else {
            output.push(character);
            in_whitespace = false;
        }
    }

    output.trim().to_string()
}

fn normalize_punctuation_spacing(text: &str) -> String {
    let mut normalized = text.to_string();
    for punctuation in [",", ".", "!", "?", ";", ":", "，", "。", "！", "？", "；", "："] {
        normalized = normalized.replace(&format!(" {punctuation}"), punctuation);
    }
    normalized
}

fn strip_leading_fillers(text: &str) -> String {
    let mut cleaned = text.trim().to_string();

    loop {
        let Some(prefix) = LEADING_FILLERS.iter().find(|prefix| cleaned.starts_with(**prefix)) else {
            break;
        };
        cleaned = cleaned[prefix.len()..].trim_start().to_string();
    }

    cleaned
}

fn collapse_duplicate_punctuation(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut previous: Option<char> = None;

    for character in text.chars() {
        let duplicate = matches!(
            (previous, character),
            (Some('.'), '.')
                | (Some(','), ',')
                | (Some('!'), '!')
                | (Some('?'), '?')
                | (Some(';'), ';')
                | (Some(':'), ':')
                | (Some('。'), '。')
                | (Some('，'), '，')
                | (Some('！'), '！')
                | (Some('？'), '？')
                | (Some('；'), '；')
                | (Some('：'), '：')
        );

        if !duplicate {
            output.push(character);
        }
        previous = Some(character);
    }

    output
}

fn standardize_cjk_punctuation(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            ',' => '，',
            ';' => '；',
            ':' => '：',
            '?' => '？',
            '!' => '！',
            _ => character,
        })
        .collect()
}

fn capitalize_first_ascii_letter(text: &str) -> String {
    let mut capitalized = String::with_capacity(text.len());
    let mut uppercased = false;

    for character in text.chars() {
        if !uppercased && character.is_ascii_alphabetic() {
            capitalized.push(character.to_ascii_uppercase());
            uppercased = true;
        } else {
            capitalized.push(character);
        }
    }

    capitalized
}

fn contains_cjk(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(
            character as u32,
            0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
        )
    })
}

fn ends_with_terminal_punctuation(text: &str) -> bool {
    text.ends_with(['.', '!', '?', '。', '！', '？'])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_output_only_trims_outer_whitespace() {
        let transformed = transform_output_text("  hello   world  ", &OutputStyle::Raw);

        assert_eq!(transformed, "hello   world");
    }

    #[test]
    fn clean_output_normalizes_whitespace_and_fillers() {
        let transformed = transform_output_text("嗯，  帮我   打开   文档  ", &OutputStyle::Clean);

        assert_eq!(transformed, "帮我 打开 文档");
    }

    #[test]
    fn formal_output_adds_terminal_punctuation_for_cjk_text() {
        let transformed = transform_output_text("嗯，帮我打开文档", &OutputStyle::Formal);

        assert_eq!(transformed, "帮我打开文档。");
    }

    #[test]
    fn formal_output_capitalizes_english_and_preserves_questions() {
        let transformed = transform_output_text("can you open the document?", &OutputStyle::Formal);

        assert_eq!(transformed, "Can you open the document?");
    }
}
