use colored::Colorize;

use crate::model::note::NoteSummary;
use crate::model::tag::TagSummary;

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
        let title = if note.title.len() > 38 {
            format!("{}…", &note.title[..37])
        } else {
            note.title.clone()
        };

        let dir = if note.dir.is_empty() {
            ".".to_string()
        } else if note.dir.len() > 13 {
            format!("{}…", &note.dir[..12])
        } else {
            note.dir.clone()
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
pub fn print_stats(stats: &serde_json::Value) {
    println!("{}", "Vault Statistics".bold().underline());
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
