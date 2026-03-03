use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::paths;
use crate::error::OvResult;
use crate::model::graph::{GraphEdge, GraphNode, VaultGraph};
use crate::model::note::Note;

/// Bidirectional link index
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LinkIndex {
    /// note_stem -> list of outgoing link targets
    pub outgoing: HashMap<String, Vec<LinkEntry>>,
    /// note_stem -> list of incoming link sources
    pub incoming: HashMap<String, Vec<LinkEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEntry {
    pub note: String,
    pub line: usize,
    pub is_embed: bool,
}

impl LinkIndex {
    /// Build link index from notes
    pub fn build(notes: &[Note]) -> Self {
        let mut outgoing: HashMap<String, Vec<LinkEntry>> = HashMap::new();
        let mut incoming: HashMap<String, Vec<LinkEntry>> = HashMap::new();

        for note in notes {
            let stem = Path::new(&note.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            for link in &note.links {
                // Outgoing from this note
                outgoing.entry(stem.clone()).or_default().push(LinkEntry {
                    note: link.target.clone(),
                    line: link.line,
                    is_embed: link.is_embed,
                });

                // Incoming to target note
                incoming
                    .entry(link.target.clone())
                    .or_default()
                    .push(LinkEntry {
                        note: stem.clone(),
                        line: link.line,
                        is_embed: link.is_embed,
                    });
            }
        }

        Self { outgoing, incoming }
    }

    /// Save link index to disk
    pub fn save(&self, vault_root: &Path) -> OvResult<()> {
        let index_dir = paths::vault_index_dir(vault_root);
        fs::create_dir_all(&index_dir)?;
        let path = index_dir.join("links.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Build full vault graph
    pub fn to_graph(&self, notes: &[Note]) -> VaultGraph {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut all_stems: HashSet<String> = HashSet::new();
        let mut degree_map: HashMap<String, (usize, usize)> = HashMap::new();

        // Build nodes from actual notes
        for note in notes {
            let stem = Path::new(&note.path)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let out_count = self.outgoing.get(&stem).map(|v| v.len()).unwrap_or(0);
            let in_count = self.incoming.get(&stem).map(|v| v.len()).unwrap_or(0);

            nodes.push(GraphNode {
                id: stem.clone(),
                title: note.title.clone(),
                dir: note.dir.clone(),
                tags: note.tags.clone(),
                link_count: out_count,
                backlink_count: in_count,
            });

            degree_map.insert(stem.clone(), (in_count, out_count));
            all_stems.insert(stem);
        }

        // Build edges
        for (source, targets) in &self.outgoing {
            for entry in targets {
                edges.push(GraphEdge {
                    source: source.clone(),
                    target: entry.note.clone(),
                    is_embed: entry.is_embed,
                });
            }
        }

        // Find orphans (notes with no links in or out)
        let orphans: Vec<String> = all_stems
            .iter()
            .filter(|s| {
                let (in_deg, out_deg) = degree_map.get(*s).unwrap_or(&(0, 0));
                *in_deg == 0 && *out_deg == 0
            })
            .cloned()
            .collect();

        VaultGraph {
            nodes,
            edges,
            orphans,
            degree_map,
        }
    }

    /// BFS subgraph from a center node
    pub fn subgraph(&self, center: &str, depth: usize) -> (Vec<String>, Vec<(String, String)>) {
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();
        let mut edges: Vec<(String, String)> = Vec::new();

        queue.push_back((center.to_string(), 0));
        visited.insert(center.to_string());

        while let Some((node, d)) = queue.pop_front() {
            if d >= depth {
                continue;
            }

            // Outgoing links
            if let Some(targets) = self.outgoing.get(&node) {
                for entry in targets {
                    edges.push((node.clone(), entry.note.clone()));
                    if visited.insert(entry.note.clone()) {
                        queue.push_back((entry.note.clone(), d + 1));
                    }
                }
            }

            // Incoming links
            if let Some(sources) = self.incoming.get(&node) {
                for entry in sources {
                    edges.push((entry.note.clone(), node.clone()));
                    if visited.insert(entry.note.clone()) {
                        queue.push_back((entry.note.clone(), d + 1));
                    }
                }
            }
        }

        let nodes: Vec<String> = visited.into_iter().collect();
        (nodes, edges)
    }
}

/// Format graph as DOT (Graphviz)
pub fn to_dot(nodes: &[String], edges: &[(String, String)]) -> String {
    let mut out = String::from("digraph vault {\n  rankdir=LR;\n  node [shape=box];\n\n");
    for node in nodes {
        out.push_str(&format!("  \"{}\";\n", node.replace('"', "\\\"")));
    }
    out.push('\n');
    for (source, target) in edges {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\";\n",
            source.replace('"', "\\\""),
            target.replace('"', "\\\"")
        ));
    }
    out.push_str("}\n");
    out
}

/// Format graph as Mermaid
pub fn to_mermaid(_nodes: &[String], edges: &[(String, String)]) -> String {
    let mut out = String::from("graph LR\n");
    for (source, target) in edges {
        let s = source.replace(' ', "_").replace('"', "");
        let t = target.replace(' ', "_").replace('"', "");
        out.push_str(&format!("  {s}[\"{source}\"] --> {t}[\"{target}\"]\n"));
    }
    out
}
