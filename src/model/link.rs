use serde::{Deserialize, Serialize};

/// A wiki-style link [[target]] or [[target|alias]]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WikiLink {
    /// Link target (note name or path)
    pub target: String,
    /// Optional display alias
    pub alias: Option<String>,
    /// Whether this is an embed ![[...]]
    pub is_embed: bool,
    /// Line number where the link appears
    pub line: usize,
}

/// A backlink: another note linking to this one
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    /// Source note that contains the link
    pub source: String,
    /// Source note path relative to vault
    pub source_path: String,
    /// Context line around the link
    pub context: Option<String>,
    /// Line number in source
    pub line: usize,
}
