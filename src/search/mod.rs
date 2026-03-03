pub mod query;

use std::path::Path;

use crate::error::OvResult;
use crate::index::reader::{self, SearchHit};

/// High-level search with prefix parsing and post-filtering
pub fn search(
    vault_root: &Path,
    raw_query: &str,
    limit: usize,
    offset: usize,
    with_snippet: bool,
) -> OvResult<Vec<SearchHit>> {
    let parsed = query::parse_query(raw_query);

    // Build tantivy query from text part
    let tantivy_query = if parsed.text.is_empty() {
        "*".to_string()
    } else {
        parsed.text.clone()
    };

    // Search with a larger limit to allow for post-filtering
    let extra_limit = limit + offset + 100;
    let mut results = reader::search(vault_root, &tantivy_query, extra_limit, 0, with_snippet)?;

    // Post-filter by prefix conditions
    if !parsed.tags.is_empty() {
        results.retain(|hit| {
            parsed
                .tags
                .iter()
                .all(|tag| hit.tags.iter().any(|t| t == tag))
        });
    }

    if !parsed.dirs.is_empty() {
        results.retain(|hit| {
            parsed
                .dirs
                .iter()
                .any(|d| hit.dir == *d || hit.dir.starts_with(&format!("{d}/")))
        });
    }

    if !parsed.titles.is_empty() {
        results.retain(|hit| {
            parsed
                .titles
                .iter()
                .any(|t| hit.title.to_lowercase().contains(&t.to_lowercase()))
        });
    }

    if !parsed.dates.is_empty() {
        results.retain(|hit| {
            parsed
                .dates
                .iter()
                .any(|d| hit.modified.starts_with(d))
        });
    }

    if !parsed.types.is_empty() {
        results.retain(|hit| {
            parsed
                .types
                .iter()
                .any(|t| hit.note_type.eq_ignore_ascii_case(t))
        });
    }

    // Apply offset and limit
    let results: Vec<SearchHit> = results.into_iter().skip(offset).take(limit).collect();

    Ok(results)
}
