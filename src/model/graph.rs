use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub dir: String,
    pub tags: Vec<String>,
    pub link_count: usize,
    pub backlink_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub is_embed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    /// Notes with no incoming or outgoing links
    pub orphans: Vec<String>,
    /// node_id -> (in_degree, out_degree)
    pub degree_map: HashMap<String, (usize, usize)>,
}
