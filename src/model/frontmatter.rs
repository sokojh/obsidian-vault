use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// How the frontmatter was parsed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FrontmatterFormat {
    /// Standard YAML `---` delimited block
    StandardYaml,
    /// Zettelkasten: no YAML delimiters, inline Status/Tags at top
    Zettelkasten,
    /// Clippings: YAML block with template residue + inline fields outside block
    Clippings,
    /// No frontmatter detected
    None,
}

/// Parsed frontmatter from a note
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    pub format: Option<FrontmatterFormat>,
    /// Zettelkasten ID (timestamp like 202212091326)
    pub zettel_id: Option<String>,
    /// Status field (e.g., "#idea", "#clippings")
    pub status: Option<String>,
    /// Tags from frontmatter (e.g., ["#imweb", "#devops"])
    pub tags: Vec<String>,
    /// Title (from YAML title field or Clippings)
    pub title: Option<String>,
    /// Author (from Clippings)
    pub author: Option<String>,
    /// Source URL or reference
    pub source: Option<String>,
    /// Clipped date
    pub clipped: Option<String>,
    /// Any additional YAML fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
