use serde::{Deserialize, Serialize};

/// A tag found in the vault
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag {
    /// Tag name without # prefix
    pub name: String,
    /// Original form with #
    pub raw: String,
}

impl Tag {
    pub fn new(raw: &str) -> Self {
        let name = raw.strip_prefix('#').unwrap_or(raw).to_string();
        Self {
            raw: if raw.starts_with('#') {
                raw.to_string()
            } else {
                format!("#{raw}")
            },
            name,
        }
    }
}

/// Tag with usage count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub tag: String,
    pub count: usize,
    pub notes: Vec<String>,
}
