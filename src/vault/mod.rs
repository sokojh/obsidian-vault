pub mod config;
pub mod scanner;

use std::path::{Path, PathBuf};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::error::{OvError, OvResult};
use crate::extract;
use crate::model::note::Note;

pub struct Vault {
    pub root: PathBuf,
    pub obsidian_config: config::ObsidianConfig,
    files: Vec<PathBuf>,
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
        })
    }

    /// Get all scanned markdown file paths
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Number of markdown files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Re-scan the vault
    pub fn rescan(&mut self) {
        self.files = scanner::scan_vault(&self.root, &[]);
    }

    /// Read and parse a single note by path (relative to vault root)
    pub fn read_note(&self, relative_path: &str) -> OvResult<Note> {
        let full_path = self.root.join(relative_path);
        if !full_path.exists() {
            return Err(OvError::NoteNotFound(relative_path.to_string()));
        }
        extract::extract_note(&self.root, &full_path)
    }

    /// Read all notes (can be expensive for large vaults)
    pub fn read_all_notes(&self) -> Vec<Note> {
        self.files
            .iter()
            .filter_map(|f| extract::extract_note(&self.root, f).ok())
            .collect()
    }

    /// Resolve a note name to a path using fuzzy matching
    pub fn resolve_note(&self, query: &str) -> OvResult<PathBuf> {
        // 1. Exact match by filename (without .md)
        for file in &self.files {
            let stem = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
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

        // 3. Fuzzy match
        let matcher = SkimMatcherV2::default();
        let mut best_match: Option<(i64, &PathBuf)> = None;

        for file in &self.files {
            let stem = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();

            if let Some(score) = matcher.fuzzy_match(&stem, query) {
                if best_match.is_none() || score > best_match.unwrap().0 {
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
