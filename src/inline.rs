use regex::Regex;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;

use crate::{
    logger::{log_change, log_diff},
    openai::{AiRequest, OpenAI},
};

pub async fn run(folder: PathBuf, dry_run: bool, api_key: Option<String>) -> anyhow::Result<()> {
    let openai = OpenAI::new(api_key);
    let gettext_regex = create_gettext_regex();

    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        if !is_processable_file(&entry) {
            continue;
        }

        let path = entry.path();
        process_file(path, &openai, &gettext_regex, dry_run).await?;
    }

    Ok(())
}

fn create_gettext_regex() -> Regex {
    Regex::new(r#"gettext\s*\(\s*"((?:\\.|[^"\\])*)"\s*(?:,\s*[^)]*)?\)"#).unwrap()
}

fn is_processable_file(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }

    match entry.path().extension() {
        Some(ext) if ext == "ex" => true,
        _ => false,
    }
}

async fn process_file(
    path: &std::path::Path,
    openai: &OpenAI,
    regex: &Regex,
    dry_run: bool,
) -> anyhow::Result<()> {
    let content = fs::read_to_string(path)?;

    let (modified_content, changes_made) =
        translate_gettext_strings(&content, openai, regex, dry_run).await?;

    if changes_made {
        log_diff(
            path.display().to_string().as_str(),
            &content,
            &modified_content,
        );

        if !dry_run {
            fs::write(path, modified_content)?;
        }
    }

    Ok(())
}

async fn translate_gettext_strings(
    content: &str,
    openai: &OpenAI,
    regex: &Regex,
    dry_run: bool,
) -> anyhow::Result<(String, bool)> {
    let mut modified = content.to_string();
    let mut any_changes = false;

    for cap in regex.captures_iter(content) {
        let original = &cap[0];
        let text = &cap[1];

        let translation = translate_text(openai, text).await?;
        let new_text = original.replace(text, &translation);

        if original != &new_text {
            log_change(text, &translation, "INLINE", dry_run);
            any_changes = true;
            modified = modified.replace(original, &new_text);
        }
    }

    Ok((modified, any_changes))
}

async fn translate_text(openai: &OpenAI, input: &str) -> anyhow::Result<String> {
    let prompt = build_translation_prompt(input);
    let request = build_translation_request(prompt);

    openai.send(request).await
}

fn build_translation_prompt(input: &str) -> String {
    format!(
        "Translate this gettext message to English, preserving placeholders like `%{{...}}`.

		Important:
		- If it's already in English, just return the original text.
		- Just return the translation, do not add any other text or comments.

		Text to translate:
		\"{}\"",
        input
    )
}

fn build_translation_request(prompt: String) -> AiRequest {
    AiRequest::new(
		"You are a professional translator for gettext messages. You will translate the message to English. You must preserve placeholder, written in the format `%{placeholder}`.".into(),
		prompt
	)
}
