pub mod config;
pub mod scanner;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rayon::prelude::*;
use regex::Regex;

use crate::error::{OvError, OvResult};
use crate::extract;
use crate::model::note::Note;

/// Find the byte offset to insert content within a named section.
/// If the section is found, returns the position just before the next same-or-higher level heading.
/// If no such heading exists, returns content.len() (file end).
/// If the section is not found, returns content.len().
pub fn find_section_insert_point(content: &str, section: &str) -> usize {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();
    let heading_re = HEADING_RE.get_or_init(|| Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap());

    let mut section_level: Option<usize> = None;

    for cap in heading_re.captures_iter(content) {
        let level = cap[1].len();
        let heading_text = cap[2].trim();
        let match_start = cap.get(0).unwrap().start();

        if let Some(lvl) = section_level {
            if level <= lvl {
                return match_start;
            }
        } else if heading_text.eq_ignore_ascii_case(section) {
            section_level = Some(level);
        }
    }

    content.len()
}

/// Extract the content of a named section from markdown text.
/// Returns the text between the section heading and the next same-or-higher level heading.
pub fn extract_section(content: &str, section_name: &str) -> Option<String> {
    static HEADING_RE: OnceLock<Regex> = OnceLock::new();
    let heading_re = HEADING_RE.get_or_init(|| Regex::new(r"(?m)^(#{1,6})\s+(.+)$").unwrap());

    let mut section_start: Option<usize> = None;
    let mut section_level: Option<usize> = None;

    for cap in heading_re.captures_iter(content) {
        let level = cap[1].len();
        let heading_text = cap[2].trim();
        let match_start = cap.get(0).unwrap().start();
        let match_end = cap.get(0).unwrap().end();

        if let Some(lvl) = section_level {
            if level <= lvl {
                // Found the next same-or-higher level heading — section ends here
                let section_body = &content[section_start.unwrap()..match_start];
                return Some(section_body.trim().to_string());
            }
        } else if heading_text.eq_ignore_ascii_case(section_name) {
            section_level = Some(level);
            section_start = Some(match_end);
        }
    }

    // Section found but no closing heading — extends to end of content
    if let Some(start) = section_start {
        let section_body = &content[start..];
        return Some(section_body.trim().to_string());
    }

    None
}

pub struct Vault {
    pub root: PathBuf,
    pub obsidian_config: config::ObsidianConfig,
    files: Vec<PathBuf>,
    notes_cache: OnceLock<Vec<Note>>,
    skipped_count: OnceLock<usize>,
}

impl Vault {
    /// Open a vault at the given path
    pub fn open(root: PathBuf) -> OvResult<Self> {
        if !root.join(".obsidian").is_dir() {
            return Err(OvError::VaultNotFound(root.to_string_lossy().to_string()));
        }

        let obsidian_config = config::ObsidianConfig::load(&root);
        let files = scanner::scan_vault(&root, &[]);

        Ok(Self {
            root,
            obsidian_config,
            files,
            notes_cache: OnceLock::new(),
            skipped_count: OnceLock::new(),
        })
    }

    /// Get all scanned markdown file paths
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Read and parse a single note by path (relative to vault root)
    pub fn read_note(&self, relative_path: &str) -> OvResult<Note> {
        let full_path = self.root.join(relative_path);
        if !full_path.exists() {
            return Err(OvError::NoteNotFound(relative_path.to_string()));
        }
        extract::extract_note(&self.root, &full_path)
    }

    /// Get all notes, cached after first access (parallel I/O via rayon)
    pub fn notes(&self) -> &[Note] {
        self.notes_cache.get_or_init(|| {
            let results: Vec<_> = self
                .files
                .par_iter()
                .map(|f| extract::extract_note(&self.root, f))
                .collect();
            let total = results.len();
            let notes: Vec<Note> = results.into_iter().filter_map(|r| r.ok()).collect();
            let _ = self.skipped_count.set(total - notes.len());
            notes
        })
    }

    /// Number of files that failed to parse during notes() loading
    pub fn skipped_count(&self) -> usize {
        // Ensure notes are loaded first
        let _ = self.notes();
        *self.skipped_count.get().unwrap_or(&0)
    }

    /// Resolve a note name to a path.
    /// When `fuzzy` is true, falls back to fuzzy matching.
    /// When `fuzzy` is false (default for agents), only exact matches are returned.
    pub fn resolve_note_with_mode(&self, query: &str, fuzzy: bool) -> OvResult<PathBuf> {
        // 1. Exact match by filename (without .md)
        for file in &self.files {
            let stem = file.file_stem().unwrap_or_default().to_string_lossy();
            if stem.eq_ignore_ascii_case(query) {
                return Ok(file.clone());
            }
        }

        // 2. Exact match by relative path
        let query_path = if query.ends_with(".md") {
            PathBuf::from(query)
        } else {
            PathBuf::from(format!("{query}.md"))
        };
        let full = self.root.join(&query_path);
        if full.exists() {
            return Ok(full);
        }

        if !fuzzy {
            return Err(OvError::NoteNotFound(query.to_string()));
        }

        // 3. Fuzzy match (opt-in only)
        let matcher = SkimMatcherV2::default();
        let mut best_match: Option<(i64, &PathBuf)> = None;

        for file in &self.files {
            let stem = file.file_stem().unwrap_or_default().to_string_lossy();

            if let Some(score) = matcher.fuzzy_match(&stem, query) {
                if best_match.as_ref().is_none_or(|(s, _)| score > *s) {
                    best_match = Some((score, file));
                }
            }
        }

        if let Some((_, path)) = best_match {
            Ok(path.clone())
        } else {
            Err(OvError::NoteNotFound(query.to_string()))
        }
    }

    /// Get relative path for a file
    pub fn relative_path(&self, file: &Path) -> String {
        file.strip_prefix(&self.root)
            .unwrap_or(file)
            .to_string_lossy()
            .to_string()
    }

    /// Get all unique directories containing notes
    pub fn directories(&self) -> Vec<String> {
        let mut dirs: Vec<String> = self
            .files
            .iter()
            .filter_map(|f| {
                f.parent()
                    .and_then(|p| p.strip_prefix(&self.root).ok())
                    .map(|p| p.to_string_lossy().to_string())
            })
            .collect();
        dirs.sort();
        dirs.dedup();
        dirs
    }
}
