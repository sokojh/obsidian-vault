pub mod frontmatter;
pub mod patterns;

use std::path::Path;

use chrono::{DateTime, Local};

use crate::error::OvResult;
use crate::model::note::{FileMeta, Note};

/// Extract all metadata from a single file in one read
pub fn extract_note(vault_root: &Path, file_path: &Path) -> OvResult<Note> {
    let content = std::fs::read_to_string(file_path)?;
    let relative = file_path
        .strip_prefix(vault_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    let title = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let dir = file_path
        .parent()
        .and_then(|p| p.strip_prefix(vault_root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Parse frontmatter
    let (fm, body_start) = frontmatter::parse_frontmatter(&content);

    // Extract body
    let body = if body_start > 0 && body_start < content.len() {
        &content[body_start..]
    } else {
        &content
    };

    // Extract patterns from full content
    let links = patterns::extract_links(&content);
    let headings = patterns::extract_headings(&content);
    let inline_tags = patterns::extract_inline_tags(&content);

    // Merge frontmatter tags and inline tags
    let mut all_tags = fm.tags.clone();
    for tag in &inline_tags {
        if !all_tags.contains(tag) {
            all_tags.push(tag.clone());
        }
    }

    let word_count = patterns::word_count(body);

    // File metadata
    let metadata = std::fs::metadata(file_path)?;
    let modified: DateTime<Local> = metadata.modified()?.into();
    let created: Option<DateTime<Local>> = metadata.created().ok().map(|t| t.into());

    let file_meta = FileMeta {
        path: relative.clone(),
        size: metadata.len(),
        modified: modified.to_rfc3339(),
        created: created.map(|c| c.to_rfc3339()),
        hash: Some(blake3::hash(content.as_bytes()).to_hex().to_string()),
        evicted: false,
    };

    // Use frontmatter title if available, otherwise filename
    let display_title = fm.title.clone().unwrap_or(title);

    Ok(Note {
        title: display_title,
        path: relative,
        dir,
        frontmatter: fm,
        tags: all_tags,
        links,
        headings,
        word_count,
        file_meta,
        body: Some(body.to_string()),
    })
}
