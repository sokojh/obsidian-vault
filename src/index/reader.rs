use std::path::Path;

use tantivy::collector::{Count, TopDocs};
use tantivy::query::{AllQuery, QueryParser};
use tantivy::{Index, ReloadPolicy, SnippetGenerator};

use crate::config::paths;
use crate::error::{OvError, OvResult};
use crate::model::note::NoteSummary;

use super::tokenizer;

/// A search hit
#[derive(serde::Serialize)]
pub struct SearchHit {
    pub path: String,
    pub title: String,
    pub dir: String,
    pub tags: Vec<String>,
    pub modified: String,
    pub score: f32,
    pub snippet: Option<String>,
    pub note_type: String,
}

/// Open the index for reading and execute a query
pub fn search(
    vault_root: &Path,
    query_str: &str,
    limit: usize,
    offset: usize,
    with_snippet: bool,
) -> OvResult<Vec<SearchHit>> {
    let index_dir = paths::vault_index_dir(vault_root);
    let tantivy_dir = index_dir.join("tantivy");

    if !tantivy_dir.exists() {
        return Err(OvError::IndexNotBuilt);
    }

    let (_schema, fields) = super::schema::build_schema();
    let index = Index::open_in_dir(&tantivy_dir).map_err(|e| OvError::General(e.to_string()))?;

    // Register tokenizer
    index.tokenizers().register(
        tokenizer::tokenizer_name(),
        tokenizer::build_text_analyzer(),
    );

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
        .map_err(|e: tantivy::TantivyError| OvError::General(e.to_string()))?;

    let searcher = reader.searcher();

    // Build query parser searching title and body
    let mut query_parser = QueryParser::for_index(&index, vec![fields.title, fields.body]);
    query_parser.set_field_boost(fields.title, 3.0);

    let query = query_parser
        .parse_query(query_str)
        .map_err(|e| OvError::QueryParse(e.to_string()))?;

    let top_docs = searcher
        .search(&query, &TopDocs::with_limit(limit + offset))
        .map_err(|e| OvError::General(e.to_string()))?;

    // Build snippet generator if needed
    let snippet_gen = if with_snippet {
        Some(
            SnippetGenerator::create(&searcher, &query, fields.body)
                .map_err(|e| OvError::General(e.to_string()))?,
        )
    } else {
        None
    };

    let mut results = Vec::new();
    for (i, (score, doc_addr)) in top_docs.iter().enumerate() {
        if i < offset {
            continue;
        }

        let doc: tantivy::TantivyDocument = searcher
            .doc(*doc_addr)
            .map_err(|e| OvError::General(e.to_string()))?;

        let path = get_field_text(&doc, &fields.path);
        let title = get_field_text(&doc, &fields.title);
        let dir = get_field_text(&doc, &fields.dir);
        let modified = get_field_text(&doc, &fields.modified);

        let tags: Vec<String> = doc
            .get_all(fields.tags)
            .filter_map(|v| {
                if let tantivy::schema::OwnedValue::Str(s) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        let snippet = snippet_gen.as_ref().map(|gen| {
            let snippet = gen.snippet_from_doc(&doc);
            snippet.to_html()
        });

        let note_type = get_field_text(&doc, &fields.note_type);

        results.push(SearchHit {
            path,
            title,
            dir,
            tags,
            modified,
            score: *score,
            snippet,
            note_type,
        });
    }

    Ok(results)
}

fn get_field_text(doc: &tantivy::TantivyDocument, field: &tantivy::schema::Field) -> String {
    doc.get_first(*field)
        .and_then(|v| {
            if let tantivy::schema::OwnedValue::Str(s) = v {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn get_field_u64(doc: &tantivy::TantivyDocument, field: &tantivy::schema::Field) -> u64 {
    doc.get_first(*field)
        .and_then(|v| {
            if let tantivy::schema::OwnedValue::U64(n) = v {
                Some(*n)
            } else {
                None
            }
        })
        .unwrap_or(0)
}

/// Read all note summaries directly from the Tantivy index (no file I/O).
/// Returns None if the index doesn't exist or is incompatible.
pub fn read_all_from_index(vault_root: &Path) -> Option<Vec<NoteSummary>> {
    let index_dir = paths::vault_index_dir(vault_root);
    let tantivy_dir = index_dir.join("tantivy");

    if !tantivy_dir.exists() {
        return None;
    }

    let index = Index::open_in_dir(&tantivy_dir).ok()?;

    // Check schema has word_count field (v2 schema)
    if index.schema().get_field("word_count").is_err() {
        return None;
    }

    let (_schema, fields) = super::schema::build_schema();

    index.tokenizers().register(
        tokenizer::tokenizer_name(),
        tokenizer::build_text_analyzer(),
    );

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
        .ok()?;

    let searcher = reader.searcher();

    // Count total docs
    let total = searcher.search(&AllQuery, &Count).ok()?;
    if total == 0 {
        return Some(Vec::new());
    }

    // Retrieve all documents
    let top_docs = searcher
        .search(&AllQuery, &TopDocs::with_limit(total))
        .ok()?;

    let mut summaries = Vec::with_capacity(top_docs.len());
    for (_score, doc_addr) in &top_docs {
        let doc: tantivy::TantivyDocument = searcher.doc(*doc_addr).ok()?;

        let tags: Vec<String> = doc
            .get_all(fields.tags)
            .filter_map(|v| {
                if let tantivy::schema::OwnedValue::Str(s) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        summaries.push(NoteSummary {
            title: get_field_text(&doc, &fields.title),
            path: get_field_text(&doc, &fields.path),
            dir: get_field_text(&doc, &fields.dir),
            tags,
            modified: get_field_text(&doc, &fields.modified),
            word_count: get_field_u64(&doc, &fields.word_count) as usize,
            link_count: get_field_u64(&doc, &fields.link_count) as usize,
            evicted: false,
        });
    }

    Some(summaries)
}
