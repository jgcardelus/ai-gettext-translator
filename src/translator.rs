use crate::logger::{log_change, log_file_success, log_no_changes};
use crate::openai::{AiRequest, OpenAI};
use anyhow::Result;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn run(
    root: PathBuf,
    langs: &str,
    dry_run: bool,
    force: bool,
    api_key: Option<String>,
) -> Result<()> {
    let openai = OpenAI::new(api_key);
    let lang_list: Vec<&str> = langs.split(',').map(|s| s.trim()).collect();

    for lang in lang_list {
        let lang_path = root.join(lang);
        if !lang_path.exists() {
            eprintln!(
                "{} {} folder not found. Skipping.",
                "⚠️".yellow(),
                lang_path.display()
            );
            continue;
        }

        for entry in WalkDir::new(lang_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|e| e == "po").unwrap_or(false) {
                process_po_file(&openai, &path, lang, dry_run, force).await?;
            }
        }
    }

    Ok(())
}

/// Process a single .po file: read it, translate missing strings, write or dry-run
async fn process_po_file(
    openai: &OpenAI,
    path: &Path,
    lang: &str,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let mut changes = 0;
    let mut i = 0;

    while i < lines.len() {
        if is_msgid(&lines[i]) {
            if let Some(result) =
                try_translate_singular(&mut lines, i, openai, lang, dry_run, force).await?
            {
                changes += result;
                i += 2;
            } else if let Some(result) =
                try_translate_plural(&mut lines, i, openai, lang, dry_run, force).await?
            {
                changes += result;
                i += 4;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    if changes > 0 {
        log_file_success(
            lang.to_uppercase().as_str(),
            changes,
            path.display().to_string().as_str(),
            dry_run,
        );

        if !dry_run {
            fs::write(path, lines.join("\n"))?;
        }
    } else {
        log_no_changes(lang, path.display().to_string().as_str());
    }

    Ok(())
}

/// Returns true if the line is a msgid declaration
fn is_msgid(line: &str) -> bool {
    line.starts_with("msgid ")
}

/// Attempts to translate a singular msgid + empty msgstr
/// Returns Some(1) if translation was done, None if not matched
async fn try_translate_singular(
    lines: &mut [String],
    i: usize,
    openai: &OpenAI,
    lang: &str,
    dry_run: bool,
    force: bool,
) -> Result<Option<usize>> {
    if i + 1 >= lines.len() || !lines[i + 1].starts_with("msgstr") {
        return Ok(None);
    }

    let msgid = extract_po_string(&lines[i])?;
    let msgstr = extract_po_string(&lines[i + 1])?;

    if msgstr.is_empty() || force {
        let translated = translate_msg(openai, &msgid, lang).await?;
        log_change(&msgid, &translated, lang, dry_run);
        lines[i + 1] = format!("msgstr \"{}\"", translated);
        Ok(Some(1))
    } else {
        Ok(None)
    }
}

/// Attempts to translate a plural msgid + msgid_plural + msgstr[0,1]
/// Returns Some(1) if translation was done, None if not matched
async fn try_translate_plural(
    lines: &mut [String],
    i: usize,
    openai: &OpenAI,
    lang: &str,
    dry_run: bool,
    force: bool,
) -> Result<Option<usize>> {
    if i + 3 >= lines.len() {
        return Ok(None);
    }

    if !lines[i + 1].starts_with("msgid_plural") || !lines[i + 2].starts_with("msgstr[0]") {
        return Ok(None);
    }

    let msgid_plural = extract_po_string(&lines[i + 1])?;
    let msgstr0 = extract_po_string(&lines[i + 2])?;
    let msgstr1 = extract_po_string(&lines[i + 3])?;

    if msgstr0.is_empty() || msgstr1.is_empty() || force {
        let translated = translate_msg(openai, &msgid_plural, lang).await?;
        log_change(&msgid_plural, &translated, lang, dry_run);
        lines[i + 2] = format!("msgstr[0] \"{}\"", translated);
        lines[i + 3] = format!("msgstr[1] \"{}\"", translated);
        Ok(Some(1))
    } else {
        Ok(None)
    }
}

/// Extracts the string from a .po line like msgid "text"
fn extract_po_string(line: &str) -> Result<String> {
    let quote_start = line
        .find('"')
        .ok_or_else(|| anyhow::anyhow!("Malformed .po line: {line}"))?;
    let quote_end = line
        .rfind('"')
        .ok_or_else(|| anyhow::anyhow!("Malformed .po line: {line}"))?;

    Ok(line[quote_start + 1..quote_end].to_string())
}
async fn translate_msg(openai: &OpenAI, msg: &str, iso_code: &str) -> Result<String> {
    let language = iso_to_name(iso_code);
    let instructions = format!(
        "You are a professional translator for gettext messages. You will translate the message to {}. You must preserve placeholder, written in the format `%{{placeholder}}`.",
        language
    );
    let prompt = build_translation_prompt(msg, language);

    let req = AiRequest::new(instructions, prompt);
    openai.send(req).await
}

fn build_translation_prompt(input: &str, lang: &str) -> String {
    format!(
        "Translate this gettext message to {}, preserving placeholders like `%{{...}}`.

		Important:
		- If it's already in {}, just return the original text.
		- Just return the translation, do not add any other text or comments.

		Text to translate:
		\"{}\"",
        lang, lang, input
    )
}

fn iso_to_name(code: &str) -> &'static str {
    match code {
        "en" => "English",
        "es" => "Spanish",
        "it" => "Italian",
        "fr" => "French",
        "de" => "German",
        "pt" => "Portuguese",
        "ja" => "Japanese",
        "zh" => "Chinese",
        "ru" => "Russian",
        _ => "English", // fallback
    }
}
