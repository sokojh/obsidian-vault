use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::Serialize;

use crate::index::link_index::LinkIndex;
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
        "title" => summaries.sort_by_key(|a| a.title.to_lowercase()),
        "size" | "words" => summaries.sort_by_key(|a| std::cmp::Reverse(a.word_count)),
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
        _ => summaries.sort_by_key(|s| std::cmp::Reverse(s.count)),
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
        avg_words_per_note: total_words.checked_div(total_notes).unwrap_or(0),
        avg_links_per_note: total_links.checked_div(total_notes).unwrap_or(0),
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
        avg_words_per_note: total_words.checked_div(total_notes).unwrap_or(0),
        avg_links_per_note: total_links.checked_div(total_notes).unwrap_or(0),
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

// ─── diagnose ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OrphanNote {
    pub title: String,
    pub path: String,
    pub dir: String,
    pub modified: String,
    pub word_count: usize,
}

/// Notes with zero incoming links (no backlinks). Optionally exclude given dirs.
pub fn find_orphans(
    notes: &[Note],
    link_idx: &LinkIndex,
    exclude_dirs: &[String],
) -> Vec<OrphanNote> {
    let mut out = Vec::new();
    for note in notes {
        if exclude_dirs.iter().any(|d| note.dir.starts_with(d)) {
            continue;
        }
        let stem = Path::new(&note.path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let incoming_count = link_idx.incoming.get(&stem).map(|v| v.len()).unwrap_or(0);
        if incoming_count == 0 {
            out.push(OrphanNote {
                title: note.title.clone(),
                path: note.path.clone(),
                dir: note.dir.clone(),
                modified: note.file_meta.modified.clone(),
                word_count: note.word_count,
            });
        }
    }
    out
}

#[derive(Debug, Serialize)]
pub struct StubNote {
    pub title: String,
    pub path: String,
    pub dir: String,
    pub modified: String,
    pub word_count: usize,
}

/// Notes whose body word count is below `max_words`. Optionally restrict by dir.
pub fn find_stubs(notes: &[Note], max_words: usize, dir: Option<&str>) -> Vec<StubNote> {
    notes
        .iter()
        .filter(|n| n.word_count <= max_words)
        .filter(|n| dir.is_none_or(|d| n.dir.starts_with(d)))
        .map(|n| StubNote {
            title: n.title.clone(),
            path: n.path.clone(),
            dir: n.dir.clone(),
            modified: n.file_meta.modified.clone(),
            word_count: n.word_count,
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct DeadLink {
    pub source: String,
    pub source_path: String,
    pub target: String,
    pub line: usize,
}

/// `[[wiki-link]]`s whose target doesn't exist as a note in the vault.
///
/// Match semantics (case-insensitive throughout, matching Obsidian + ov resolver):
///   • bare stem    `[[Foo]]`            → check against filename stems
///   • path-qualified `[[Folder/Foo]]`  → check against relative paths
///   • aliases      `[[Plan]]` matches a note with `aliases: [Plan]`
///   • heading anchor `[[Foo#Section]]` → strip `#…` and treat as `[[Foo]]`
pub fn find_dead_links(notes: &[Note]) -> Vec<DeadLink> {
    let known_stems_ci: HashSet<String> = notes
        .iter()
        .map(|n| {
            Path::new(&n.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase()
        })
        .collect();

    // Path-qualified targets: lowercased relative path WITHOUT .md
    let known_paths_ci: HashSet<String> = notes
        .iter()
        .map(|n| n.path.strip_suffix(".md").unwrap_or(&n.path).to_lowercase())
        .collect();

    // Aliases are case-insensitive too (Obsidian behavior)
    let known_aliases_ci: HashSet<String> = notes
        .iter()
        .flat_map(|n| n.frontmatter.aliases.iter().map(|a| a.to_lowercase()))
        .collect();

    let mut dead = Vec::new();
    for note in notes {
        for link in &note.links {
            // Strip optional heading anchor `Note#Section` before lookup
            let raw = link
                .target
                .split_once('#')
                .map(|(s, _)| s)
                .unwrap_or(&link.target);
            if raw.is_empty() {
                continue; // self-link to current heading like [[#Section]]
            }
            let target_ci = raw.to_lowercase();

            // 1. exact filename stem match
            if known_stems_ci.contains(&target_ci) {
                continue;
            }
            // 2. path-qualified target like [[Projects/Plan]] → match against relpath
            if target_ci.contains('/') && known_paths_ci.contains(&target_ci) {
                continue;
            }
            // 3. alias match
            if known_aliases_ci.contains(&target_ci) {
                continue;
            }
            dead.push(DeadLink {
                source: note.title.clone(),
                source_path: note.path.clone(),
                target: link.target.clone(),
                line: link.line,
            });
        }
    }
    dead
}

#[derive(Debug, Serialize)]
pub struct TopicSummary {
    pub topic: String,
    pub count: usize,
    pub notes: Vec<String>,
}

/// Aggregate frontmatter `topics:` field across all notes.
/// `topics` is not a first-class Frontmatter field (yet) — pulled from `extra`.
/// Counterpart to `aggregate_tags` but for the curated `topics` array.
pub fn aggregate_topics(notes: &[Note], min_count: usize) -> Vec<TopicSummary> {
    let mut counts: HashMap<String, Vec<String>> = HashMap::new();
    for note in notes {
        let Some(topics_val) = note.frontmatter.extra.get("topics") else {
            continue;
        };
        let serde_json::Value::Array(arr) = topics_val else {
            continue;
        };
        for item in arr {
            if let serde_json::Value::String(topic) = item {
                counts
                    .entry(topic.clone())
                    .or_default()
                    .push(note.title.clone());
            }
        }
    }
    let mut summaries: Vec<TopicSummary> = counts
        .into_iter()
        .filter(|(_, notes)| notes.len() >= min_count)
        .map(|(topic, notes)| TopicSummary {
            count: notes.len(),
            topic,
            notes,
        })
        .collect();
    // Sort by count desc, then topic asc
    summaries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.topic.cmp(&b.topic)));
    summaries
}
