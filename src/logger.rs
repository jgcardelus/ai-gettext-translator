use chrono::Local;
use colored::*;

/// Logs a change made to a gettext string
pub fn log_change(original: &str, translated: &str, lang: &str, dry_run: bool) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();

    if dry_run {
        println!(
            "{} {} [{}] \"{}\" ➜ \"{}\"",
            timestamp.dimmed(),
            "🔍".cyan(),
            lang.to_uppercase().blue(),
            original,
            translated
        );
    } else {
        println!(
            "{} {} [{}] \"{}\" ➜ \"{}\"",
            timestamp.dimmed(),
            "✏️".yellow(),
            lang.to_uppercase().blue(),
            original,
            translated
        );
    }
}

/// Logs a successful file translation
pub fn log_file_success(lang: &str, count: usize, path: &str, dry_run: bool) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();

    if dry_run {
        println!(
            "{} {} {} → would update {} entries in {}",
            timestamp.dimmed(),
            "💡".cyan(),
            lang.to_uppercase(),
            count,
            path
        );
    } else {
        println!(
            "{} {} {} → updated {} entries in {}",
            timestamp.dimmed(),
            "✅".green(),
            lang.to_uppercase(),
            count,
            path
        );
    }
}

/// Logs that a file is already complete
pub fn log_no_changes(lang: &str, path: &str) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();
    println!(
        "{} {} {} has no missing translations in {}",
        timestamp.dimmed(),
        "🟢".bright_green(),
        lang,
        path
    );
}

/// Logs a retry attempt with exponential backoff
pub fn log_retry(attempt: u32, max: u32, error: &str) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();
    println!(
        "{} {} Retry {}/{} after error: {}",
        timestamp.dimmed(),
        "🔁".yellow(),
        attempt,
        max,
        error
    );
}

/// Logs the difference between the original and modified content
pub fn log_diff(path: &str, original: &str, modified: &str) {
    let timestamp = Local::now().format("[%Y-%m-%d %H:%M:%S]").to_string();

    println!("{} {} Diff for {}:", timestamp.dimmed(), "📝".blue(), path);

    for (line_number, (old_line, new_line)) in original.lines().zip(modified.lines()).enumerate() {
        if old_line != new_line {
            println!("  {} Line {}:", "🔄".purple(), line_number + 1);
            println!("    {}", format!("- {}", old_line).red());
            println!("    {}", format!("+ {}", new_line).green());
        }
    }

    println!();
}
