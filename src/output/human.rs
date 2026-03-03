use colored::Colorize;

use crate::model::note::NoteSummary;
use crate::model::tag::TagSummary;
use crate::service::VaultStats;

/// Truncate a string to `max_chars` unicode characters, appending "…" if truncated.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut char_iter = s.char_indices();
    // Advance max_chars times; if there's a char beyond that, we need truncation
    if let Some((byte_pos, _)) = char_iter.nth(max_chars) {
        format!("{}…", s[..byte_pos].trim_end())
    } else {
        s.to_string()
    }
}

/// Print a list of notes in human-readable table format
pub fn print_note_list(notes: &[NoteSummary]) {
    if notes.is_empty() {
        println!("{}", "No notes found.".dimmed());
        return;
    }

    // Header
    println!(
        "{:<40} {:<15} {:<6} {:<8} {}",
        "Title".bold(),
        "Directory".bold(),
        "Words".bold(),
        "Links".bold(),
        "Tags".bold()
    );
    println!("{}", "─".repeat(90).dimmed());

    for note in notes {
        let title = truncate_str(&note.title, 38);

        let dir = if note.dir.is_empty() {
            ".".to_string()
        } else {
            truncate_str(&note.dir, 13)
        };

        let tags = if note.tags.is_empty() {
            String::new()
        } else {
            note.tags.join(" ")
        };

        let evicted_marker = if note.evicted { " ☁" } else { "" };

        println!(
            "{:<40} {:<15} {:<6} {:<8} {}{}",
            title.cyan(),
            dir.dimmed(),
            note.word_count,
            note.link_count,
            tags.yellow(),
            evicted_marker.dimmed()
        );
    }

    println!("\n{} notes", notes.len());
}

/// Print tag summary in human-readable format
pub fn print_tag_list(tags: &[TagSummary]) {
    if tags.is_empty() {
        println!("{}", "No tags found.".dimmed());
        return;
    }

    println!("{:<30} {}", "Tag".bold(), "Count".bold());
    println!("{}", "─".repeat(40).dimmed());

    for tag in tags {
        println!("{:<30} {}", tag.tag.yellow(), tag.count);
    }

    println!("\n{} unique tags", tags.len());
}

/// Print vault statistics in human-readable format
pub fn print_stats(stats: &VaultStats) {
    println!("{}", "Vault Statistics".bold().underline());
    println!();

    println!("  {:<25} {}", "total notes".dimmed(), stats.total_notes.to_string().cyan());
    println!("  {:<25} {}", "total words".dimmed(), stats.total_words.to_string().cyan());
    println!("  {:<25} {}", "total links".dimmed(), stats.total_links.to_string().cyan());
    println!("  {:<25} {}", "unique tags".dimmed(), stats.unique_tags.to_string().cyan());
    println!("  {:<25} {}", "directories".dimmed(), stats.directories.to_string().cyan());
    println!("  {:<25} {}", "total size bytes".dimmed(), stats.total_size_bytes.to_string().cyan());
    println!("  {:<25} {}", "total size mb".dimmed(), stats.total_size_mb.cyan());
    println!("  {:<25} {}", "evicted files".dimmed(), stats.evicted_files.to_string().cyan());
    println!("  {:<25} {}", "avg words per note".dimmed(), stats.avg_words_per_note.to_string().cyan());
    println!("  {:<25} {}", "avg links per note".dimmed(), stats.avg_links_per_note.to_string().cyan());

    if !stats.top_tags.is_empty() {
        println!("  {}:", "top tags".dimmed());
        for tc in &stats.top_tags {
            println!("    - {} ({})", tc.tag.yellow(), tc.count);
        }
    }

    if !stats.directory_list.is_empty() {
        println!("  {}:", "directory list".dimmed());
        for dir in &stats.directory_list {
            println!("    - {}", dir.yellow());
        }
    }
}

/// Print generic JSON stats in human-readable format (for index status, etc.)
pub fn print_json_stats(stats: &serde_json::Value) {
    println!("{}", "Statistics".bold().underline());
    println!();

    if let Some(obj) = stats.as_object() {
        for (key, value) in obj {
            let label = key.replace('_', " ");
            match value {
                serde_json::Value::Number(n) => {
                    println!("  {:<25} {}", label.dimmed(), n.to_string().cyan());
                }
                serde_json::Value::String(s) => {
                    println!("  {:<25} {}", label.dimmed(), s.cyan());
                }
                serde_json::Value::Array(arr) => {
                    println!("  {}:", label.dimmed());
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            println!("    - {}", s.yellow());
                        }
                    }
                }
                _ => {
                    println!("  {:<25} {}", label.dimmed(), value);
                }
            }
        }
    }
}

/// Print a single note in human-readable format
pub fn print_note_detail(title: &str, path: &str, tags: &[String], body: &str) {
    println!("{}", title.bold().cyan());
    println!("{}", path.dimmed());
    if !tags.is_empty() {
        println!("Tags: {}", tags.join(" ").yellow());
    }
    println!("{}", "─".repeat(60).dimmed());
    println!("{body}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_ascii_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_ascii_exact() {
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_ascii_long() {
        let result = truncate_str("hello world!", 5);
        assert_eq!(result, "hello…");
    }

    #[test]
    fn test_truncate_korean() {
        // Korean chars are 3 bytes each in UTF-8; byte slicing would panic
        let korean = "한국어로된제목입니다아주긴노트제목";
        let result = truncate_str(korean, 5);
        assert_eq!(result, "한국어로된…");
    }

    #[test]
    fn test_truncate_mixed_cjk_latin() {
        let mixed = "Hello한국어World";
        let result = truncate_str(mixed, 8);
        assert_eq!(result, "Hello한국어…");
    }

    #[test]
    fn test_truncate_emoji() {
        let emoji = "🎉🎊🎈🎁🎆🎇✨";
        let result = truncate_str(emoji, 3);
        assert_eq!(result, "🎉🎊🎈…");
    }

    #[test]
    fn test_truncate_empty() {
        assert_eq!(truncate_str("", 10), "");
    }

    #[test]
    fn test_truncate_trailing_space_trimmed() {
        let result = truncate_str("hello   world", 6);
        // After taking 6 chars: "hello " → trim_end → "hello"
        assert_eq!(result, "hello…");
    }
}
