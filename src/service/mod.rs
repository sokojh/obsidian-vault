use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::model::link::Backlink;
use crate::model::note::{Note, NoteSummary};
use crate::model::tag::TagSummary;
use crate::vault::Vault;

// ─── list ────────────────────────────────────────────────────────────────

pub struct ListParams {
    pub dir: Option<String>,
    pub tag: Option<String>,
    pub date: Option<String>,
    pub sort: String,
    pub reverse: bool,
    pub limit: usize,
    pub offset: usize,
}

pub struct ListResult {
    pub notes: Vec<NoteSummary>,
    pub total: usize,
}

/// List notes from pre-built NoteSummary slice (index-first path)
pub fn list_summaries(summaries: &[NoteSummary], params: &ListParams) -> ListResult {
    let mut summaries: Vec<NoteSummary> = summaries.to_vec();
    apply_list_filters(&mut summaries, params);
    let total = summaries.len();
    let notes = summaries
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .collect();
    ListResult { notes, total }
}

pub fn list_notes(notes: &[Note], params: &ListParams) -> ListResult {
    let mut summaries: Vec<NoteSummary> = notes.iter().map(NoteSummary::from).collect();
    apply_list_filters(&mut summaries, params);
    let total = summaries.len();
    let notes = summaries
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .collect();
    ListResult { notes, total }
}

fn apply_list_filters(summaries: &mut Vec<NoteSummary>, params: &ListParams) {
    if let Some(ref dir) = params.dir {
        summaries.retain(|n| n.dir == *dir || n.dir.starts_with(&format!("{dir}/")));
    }
    if let Some(ref tag) = params.tag {
        let tag_normalized = if tag.starts_with('#') {
            tag.clone()
        } else {
            format!("#{tag}")
        };
        summaries.retain(|n| n.tags.iter().any(|t| t == &tag_normalized));
    }
    if let Some(ref date) = params.date {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let date_filter = match date.as_str() {
            "today" => today,
            _ => date.clone(),
        };
        summaries.retain(|n| n.modified.starts_with(&date_filter));
    }
    match params.sort.as_str() {
        "title" => summaries.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
        "size" | "words" => summaries.sort_by(|a, b| b.word_count.cmp(&a.word_count)),
        _ => summaries.sort_by(|a, b| b.modified.cmp(&a.modified)),
    }
    if params.reverse {
        summaries.reverse();
    }
}

// ─── tags ────────────────────────────────────────────────────────────────

pub struct TagsParams {
    pub sort: String,
    pub min_count: Option<usize>,
    pub limit: Option<usize>,
}

/// Aggregate tags from pre-built summaries (index-first path)
pub fn aggregate_tags_from_summaries(
    summaries: &[NoteSummary],
    params: &TagsParams,
) -> Vec<TagSummary> {
    let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
    for s in summaries {
        for tag in &s.tags {
            tag_map
                .entry(tag.clone())
                .or_default()
                .push(s.title.clone());
        }
    }
    finish_aggregate_tags(tag_map, params)
}

pub fn aggregate_tags(notes: &[Note], params: &TagsParams) -> Vec<TagSummary> {
    let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
    for note in notes {
        for tag in &note.tags {
            tag_map
                .entry(tag.clone())
                .or_default()
                .push(note.title.clone());
        }
    }
    finish_aggregate_tags(tag_map, params)
}

fn finish_aggregate_tags(
    tag_map: HashMap<String, Vec<String>>,
    params: &TagsParams,
) -> Vec<TagSummary> {
    let mut summaries: Vec<TagSummary> = tag_map
        .into_iter()
        .map(|(tag, notes)| TagSummary {
            count: notes.len(),
            tag,
            notes,
        })
        .collect();

    if let Some(min) = params.min_count {
        summaries.retain(|s| s.count >= min);
    }

    match params.sort.as_str() {
        "name" => summaries.sort_by(|a, b| a.tag.cmp(&b.tag)),
        _ => summaries.sort_by(|a, b| b.count.cmp(&a.count)),
    }

    if let Some(limit) = params.limit {
        summaries.truncate(limit);
    }

    summaries
}

// ─── stats ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct TagCount {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultStats {
    pub total_notes: usize,
    pub total_words: usize,
    pub total_links: usize,
    pub unique_tags: usize,
    pub directories: usize,
    pub total_size_bytes: u64,
    pub total_size_mb: String,
    pub evicted_files: usize,
    pub skipped_files: usize,
    pub avg_words_per_note: usize,
    pub avg_links_per_note: usize,
    pub top_tags: Vec<TagCount>,
    pub directory_list: Vec<String>,
    pub source: String,
}

/// Compute stats from pre-built summaries (index-first path, no file I/O)
pub fn compute_stats_from_summaries(dirs: Vec<String>, summaries: &[NoteSummary]) -> VaultStats {
    let total_notes = summaries.len();
    let total_words: usize = summaries.iter().map(|s| s.word_count).sum();
    let total_links: usize = summaries.iter().map(|s| s.link_count).sum();

    let mut all_tags: HashMap<String, usize> = HashMap::new();
    for s in summaries {
        for tag in &s.tags {
            *all_tags.entry(tag.clone()).or_default() += 1;
        }
    }

    let mut sorted_tags: Vec<_> = all_tags.iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(a.1));
    let top_tags: Vec<TagCount> = sorted_tags
        .iter()
        .take(10)
        .map(|(tag, count)| TagCount {
            tag: (*tag).clone(),
            count: **count,
        })
        .collect();

    VaultStats {
        total_notes,
        total_words,
        total_links,
        unique_tags: all_tags.len(),
        directories: dirs.len(),
        total_size_bytes: 0, // not available from index
        total_size_mb: "N/A".to_string(),
        evicted_files: 0,
        skipped_files: 0,
        avg_words_per_note: if total_notes > 0 {
            total_words / total_notes
        } else {
            0
        },
        avg_links_per_note: if total_notes > 0 {
            total_links / total_notes
        } else {
            0
        },
        top_tags,
        directory_list: dirs,
        source: "index".to_string(),
    }
}

pub fn compute_stats(vault: &Vault, notes: &[Note]) -> VaultStats {
    let total_notes = notes.len();
    let total_words: usize = notes.iter().map(|n| n.word_count).sum();
    let total_links: usize = notes.iter().map(|n| n.links.len()).sum();

    let mut all_tags: HashMap<String, usize> = HashMap::new();
    for note in notes {
        for tag in &note.tags {
            *all_tags.entry(tag.clone()).or_default() += 1;
        }
    }

    let dirs = vault.directories();
    let total_size: u64 = notes.iter().map(|n| n.file_meta.size).sum();
    let evicted = notes.iter().filter(|n| n.file_meta.evicted).count();

    let mut sorted_tags: Vec<_> = all_tags.iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(a.1));
    let top_tags: Vec<TagCount> = sorted_tags
        .iter()
        .take(10)
        .map(|(tag, count)| TagCount {
            tag: (*tag).clone(),
            count: **count,
        })
        .collect();

    VaultStats {
        total_notes,
        total_words,
        total_links,
        unique_tags: all_tags.len(),
        directories: dirs.len(),
        total_size_bytes: total_size,
        total_size_mb: format!("{:.1}", total_size as f64 / 1_048_576.0),
        evicted_files: evicted,
        skipped_files: vault.skipped_count(),
        avg_words_per_note: if total_notes > 0 {
            total_words / total_notes
        } else {
            0
        },
        avg_links_per_note: if total_notes > 0 {
            total_links / total_notes
        } else {
            0
        },
        top_tags,
        directory_list: dirs,
        source: "full_scan".to_string(),
    }
}

// ─── backlinks ───────────────────────────────────────────────────────────

/// Find backlinks to a target note.
///
/// `vault_root` is needed for context extraction: link line numbers reference
/// the full file (including frontmatter), so we re-read from disk for accuracy.
pub fn find_backlinks(
    vault_root: &Path,
    target_stem: &str,
    notes: &[Note],
    context: bool,
) -> Vec<Backlink> {
    let mut backlinks = Vec::new();

    for note in notes {
        for link in &note.links {
            if link.target.eq_ignore_ascii_case(target_stem) {
                let ctx = if context {
                    let full_path = vault_root.join(&note.path);
                    std::fs::read_to_string(&full_path)
                        .ok()
                        .and_then(|content| {
                            content
                                .lines()
                                .nth(link.line.saturating_sub(1))
                                .map(|l| l.trim().to_string())
                        })
                } else {
                    None
                };

                backlinks.push(Backlink {
                    source: note.title.clone(),
                    source_path: note.path.clone(),
                    context: ctx,
                    line: link.line,
                });
            }
        }
    }

    backlinks
}
