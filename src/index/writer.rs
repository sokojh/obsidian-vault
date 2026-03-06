use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

use tantivy::{Index, IndexWriter};

use crate::config::paths;
use crate::error::{OvError, OvResult};
use crate::extract;
use crate::vault::Vault;

use super::tokenizer;

/// File metadata for incremental indexing
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct FileHashEntry {
    size: u64,
    modified: String,
    hash: String,
}

/// Index metadata
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct IndexMeta {
    vault_path: String,
    last_build: String,
    total_docs: usize,
    build_time_ms: u64,
}

/// Build or update the search index
pub fn build_index(vault: &Vault, force: bool) -> OvResult<BuildResult> {
    let start = Instant::now();
    let index_dir = paths::vault_index_dir(&vault.root);
    let tantivy_dir = index_dir.join("tantivy");
    let hashes_path = index_dir.join("file_hashes.json");
    let meta_path = index_dir.join("meta.json");

    fs::create_dir_all(&tantivy_dir)?;

    // Load existing file hashes for incremental update
    let old_hashes: HashMap<String, FileHashEntry> = if !force && hashes_path.exists() {
        let content = fs::read_to_string(&hashes_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    let (schema, fields) = super::schema::build_schema();

    // Open or create index
    let index = if tantivy_dir.join("meta.json").exists() && !force {
        match Index::open_in_dir(&tantivy_dir) {
            Ok(idx) => {
                // Verify schema compatibility — if new fields are missing, force rebuild
                if idx.schema().get_field("word_count").is_err() {
                    // Clear both tantivy dir AND file hashes to force full reindex
                    fs::remove_dir_all(&tantivy_dir)?;
                    fs::create_dir_all(&tantivy_dir)?;
                    if hashes_path.exists() {
                        fs::remove_file(&hashes_path)?;
                    }
                    Index::create_in_dir(&tantivy_dir, schema.clone())
                        .map_err(|e| OvError::General(e.to_string()))?
                } else {
                    idx
                }
            }
            Err(_) => {
                if tantivy_dir.exists() {
                    fs::remove_dir_all(&tantivy_dir)?;
                    fs::create_dir_all(&tantivy_dir)?;
                }
                Index::create_in_dir(&tantivy_dir, schema.clone())
                    .map_err(|e| OvError::General(e.to_string()))?
            }
        }
    } else {
        // Clean and recreate
        if tantivy_dir.exists() {
            fs::remove_dir_all(&tantivy_dir)?;
            fs::create_dir_all(&tantivy_dir)?;
        }
        Index::create_in_dir(&tantivy_dir, schema.clone())
            .map_err(|e| OvError::General(e.to_string()))?
    };

    // Register tokenizer
    index.tokenizers().register(
        tokenizer::tokenizer_name(),
        tokenizer::build_text_analyzer(),
    );

    let mut writer: IndexWriter = index
        .writer(50_000_000)
        .map_err(|e| OvError::General(e.to_string()))?;

    let files = vault.files();
    let mut new_hashes: HashMap<String, FileHashEntry> = HashMap::new();
    let mut indexed = 0usize;
    let mut skipped = 0usize;

    // Collect current file paths for stale detection
    let current_paths: HashSet<String> = files.iter().map(|f| vault.relative_path(f)).collect();

    // Delete stale entries: files that were in the old index but no longer exist
    for old_path in old_hashes.keys() {
        if !current_paths.contains(old_path) {
            let path_term = tantivy::Term::from_field_text(fields.path, old_path);
            writer.delete_term(path_term);
        }
    }

    for file in files {
        let relative = vault.relative_path(file);

        // Check if file changed (incremental)
        if let Ok(metadata) = fs::metadata(file) {
            let modified: chrono::DateTime<chrono::Local> = match metadata.modified() {
                Ok(t) => t.into(),
                Err(_) => chrono::Local::now(),
            };
            let mod_str = modified.to_rfc3339();
            let size = metadata.len();

            if let Some(old) = old_hashes.get(&relative) {
                if old.modified == mod_str && old.size == size {
                    // File unchanged, keep old hash
                    new_hashes.insert(relative, old.clone());
                    skipped += 1;
                    continue;
                }
            }

            // Need to index this file
            if let Ok(note) = extract::extract_note(&vault.root, file) {
                let hash = note.file_meta.hash.clone().unwrap_or_default();

                // Delete old document if it exists (for updates)
                let path_term = tantivy::Term::from_field_text(fields.path, &relative);
                writer.delete_term(path_term);

                // Add new document
                let mut doc = tantivy::TantivyDocument::new();
                doc.add_text(fields.path, &relative);
                doc.add_text(fields.title, &note.title);
                doc.add_text(fields.body, note.body.as_deref().unwrap_or(""));
                for tag in &note.tags {
                    doc.add_text(fields.tags, tag);
                }
                doc.add_text(fields.dir, &note.dir);
                doc.add_text(fields.modified, &mod_str);
                doc.add_text(fields.hash, &hash);
                doc.add_u64(fields.word_count, note.word_count as u64);
                doc.add_u64(fields.file_size, size);
                doc.add_u64(fields.link_count, note.links.len() as u64);

                // Extract type from frontmatter extra
                let note_type = note
                    .frontmatter
                    .extra
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                doc.add_text(fields.note_type, note_type);

                writer
                    .add_document(doc)
                    .map_err(|e| OvError::General(e.to_string()))?;

                new_hashes.insert(
                    relative,
                    FileHashEntry {
                        size,
                        modified: mod_str,
                        hash,
                    },
                );
                indexed += 1;
            }
        }
    }

    writer
        .commit()
        .map_err(|e| OvError::General(e.to_string()))?;

    let elapsed = start.elapsed();

    // Save file hashes
    let hashes_json = serde_json::to_string_pretty(&new_hashes)?;
    fs::write(&hashes_path, hashes_json)?;

    // Save metadata
    let meta = IndexMeta {
        vault_path: vault.root.to_string_lossy().to_string(),
        last_build: chrono::Local::now().to_rfc3339(),
        total_docs: indexed + skipped,
        build_time_ms: elapsed.as_millis() as u64,
    };
    let meta_json = serde_json::to_string_pretty(&meta)?;
    fs::write(&meta_path, meta_json)?;

    Ok(BuildResult {
        indexed,
        skipped,
        total: indexed + skipped,
        elapsed_ms: elapsed.as_millis() as u64,
    })
}

/// Result of an index build
pub struct BuildResult {
    pub indexed: usize,
    pub skipped: usize,
    pub total: usize,
    pub elapsed_ms: u64,
}

/// Get index status
pub fn index_status(vault_root: &Path) -> OvResult<serde_json::Value> {
    let index_dir = paths::vault_index_dir(vault_root);
    let meta_path = index_dir.join("meta.json");

    if !meta_path.exists() {
        return Ok(serde_json::json!({
            "exists": false,
            "message": "Index not built. Run `ov index build`."
        }));
    }

    let content = fs::read_to_string(&meta_path)?;
    let meta: IndexMeta = serde_json::from_str(&content).unwrap_or_default();

    // Calculate index size
    let tantivy_dir = index_dir.join("tantivy");
    let index_size = dir_size(&tantivy_dir);

    Ok(serde_json::json!({
        "exists": true,
        "vault_path": meta.vault_path,
        "last_build": meta.last_build,
        "total_docs": meta.total_docs,
        "build_time_ms": meta.build_time_ms,
        "index_size_bytes": index_size,
        "index_size_mb": format!("{:.2}", index_size as f64 / 1_048_576.0),
        "index_path": index_dir.to_string_lossy(),
    }))
}

/// Clear the index
pub fn clear_index(vault_root: &Path) -> OvResult<()> {
    let index_dir = paths::vault_index_dir(vault_root);
    if index_dir.exists() {
        fs::remove_dir_all(&index_dir)?;
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
