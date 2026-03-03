use tantivy::schema::{
    Field, Schema, TextFieldIndexing, TextOptions, STORED, STRING,
};

use super::tokenizer;

/// All field handles for the search index
#[derive(Clone)]
pub struct IndexFields {
    pub path: Field,
    pub title: Field,
    pub body: Field,
    pub tags: Field,
    pub dir: Field,
    pub modified: Field,
    pub hash: Field,
    pub note_type: Field,
}

/// Build the tantivy schema
pub fn build_schema() -> (Schema, IndexFields) {
    let mut builder = Schema::builder();

    // Stored + indexed fields
    let path = builder.add_text_field("path", STRING | STORED);

    let title_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer::tokenizer_name())
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let title = builder.add_text_field("title", title_options);

    let body_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(tokenizer::tokenizer_name())
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let body = builder.add_text_field("body", body_options);

    let tags = builder.add_text_field("tags", STRING | STORED);
    let dir = builder.add_text_field("dir", STRING | STORED);
    let modified = builder.add_text_field("modified", STRING | STORED);
    let hash = builder.add_text_field("hash", STORED);
    let note_type = builder.add_text_field("note_type", STRING | STORED);

    let schema = builder.build();
    let fields = IndexFields {
        path,
        title,
        body,
        tags,
        dir,
        modified,
        hash,
        note_type,
    };

    (schema, fields)
}
