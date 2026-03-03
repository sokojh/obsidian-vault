use serde::{Deserialize, Serialize};

/// Tag with usage count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagSummary {
    pub tag: String,
    pub count: usize,
    pub notes: Vec<String>,
}
