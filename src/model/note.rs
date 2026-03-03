use serde::{Deserialize, Serialize};

use super::frontmatter::Frontmatter;
use super::link::WikiLink;

/// File metadata from filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    /// Relative path from vault root
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified time (RFC3339)
    pub modified: String,
    /// Created time (RFC3339)
    pub created: Option<String>,
    /// BLAKE3 hash of content
    pub hash: Option<String>,
    /// Whether file is iCloud evicted (not downloaded)
    pub evicted: bool,
}

/// Full note with all extracted metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Note title (filename without .md, or from frontmatter)
    pub title: String,
    /// Relative path from vault root
    pub path: String,
    /// Directory within vault
    pub dir: String,
    /// Parsed frontmatter
    pub frontmatter: Frontmatter,
    /// All tags found (frontmatter + inline)
    pub tags: Vec<String>,
    /// All wiki links found
    pub links: Vec<WikiLink>,
    /// Headings (level, text)
    pub headings: Vec<(u8, String)>,
    /// Word count (approximate)
    pub word_count: usize,
    /// File metadata
    pub file_meta: FileMeta,
    /// Raw body content (without frontmatter)
    pub body: Option<String>,
}

/// Lightweight note summary for list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub title: String,
    pub path: String,
    pub dir: String,
    pub tags: Vec<String>,
    pub modified: String,
    pub word_count: usize,
    pub link_count: usize,
    pub evicted: bool,
}

impl From<&Note> for NoteSummary {
    fn from(note: &Note) -> Self {
        Self {
            title: note.title.clone(),
            path: note.path.clone(),
            dir: note.dir.clone(),
            tags: note.tags.clone(),
            modified: note.file_meta.modified.clone(),
            word_count: note.word_count,
            link_count: note.links.len(),
            evicted: note.file_meta.evicted,
        }
    }
}
