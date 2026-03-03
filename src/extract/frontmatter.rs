use lazy_static::lazy_static;
use regex::Regex;

use crate::model::frontmatter::{Frontmatter, FrontmatterFormat};

lazy_static! {
    /// Standard YAML frontmatter: ---\n...\n---
    static ref YAML_BLOCK_RE: Regex =
        Regex::new(r"(?s)\A---\n(.*?)\n---").unwrap();

    /// Zettelkasten ID: 12-digit number at start of file
    static ref ZETTEL_ID_RE: Regex =
        Regex::new(r"^(\d{12})$").unwrap();

    /// Status: line (e.g., "Status: #idea" or "Status:")
    static ref STATUS_RE: Regex =
        Regex::new(r"^Status:\s*(.*)$").unwrap();

    /// Tags: line (e.g., "Tags: #imweb #devops")
    static ref TAGS_LINE_RE: Regex =
        Regex::new(r"^Tags:\s*(.*)$").unwrap();

    /// Individual tags within a Tags line
    static ref TAG_IN_LINE_RE: Regex =
        Regex::new(r"#([\w\p{Hangul}/\-]+)").unwrap();

    /// Inline field: "key: value" outside YAML block (Clippings format)
    static ref INLINE_FIELD_RE: Regex =
        Regex::new(r"^(\w+):\s*(.+)$").unwrap();

    /// Source field with markdown link: source: [title](url)
    static ref SOURCE_RE: Regex =
        Regex::new(r"^source:\s*\[.*?\]\((.*?)\)").unwrap();

    /// Connection field: 연결: → [[note]]
    static ref CONNECTION_RE: Regex =
        Regex::new(r"^연결:\s*→?\s*\[\[(.+?)\]\]").unwrap();
}

/// Parse frontmatter from file content. Returns (Frontmatter, body_start_offset)
pub fn parse_frontmatter(content: &str) -> (Frontmatter, usize) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return (Frontmatter::default(), 0);
    }

    // Try Zettelkasten format first (starts with 12-digit number, no ---)
    if let Some(fm) = try_parse_zettelkasten(&lines) {
        let body_start = find_zettelkasten_body_start(content);
        return (fm, body_start);
    }

    // Try standard YAML frontmatter
    if let Some((fm, body_start)) = try_parse_yaml_block(content, &lines) {
        return (fm, body_start);
    }

    // No frontmatter
    (
        Frontmatter {
            format: Some(FrontmatterFormat::None),
            ..Default::default()
        },
        0,
    )
}

fn try_parse_zettelkasten(lines: &[&str]) -> Option<Frontmatter> {
    if lines.is_empty() {
        return None;
    }

    // First line must be a 12-digit Zettelkasten ID
    let first_line = lines[0].trim();
    if !ZETTEL_ID_RE.is_match(first_line) {
        return None;
    }

    let mut fm = Frontmatter {
        format: Some(FrontmatterFormat::Zettelkasten),
        zettel_id: Some(first_line.to_string()),
        ..Default::default()
    };

    // Parse subsequent lines until --- or empty lines accumulate
    for line in lines.iter().skip(1) {
        let line = line.trim();

        // Body separator
        if line == "---" {
            break;
        }
        // Empty line might be part of frontmatter spacing
        if line.is_empty() {
            continue;
        }

        if let Some(cap) = STATUS_RE.captures(line) {
            fm.status = Some(cap[1].trim().to_string());
        } else if let Some(cap) = TAGS_LINE_RE.captures(line) {
            let tags_str = &cap[1];
            for tag_cap in TAG_IN_LINE_RE.captures_iter(tags_str) {
                fm.tags.push(format!("#{}", &tag_cap[1]));
            }
        }
    }

    Some(fm)
}

fn find_zettelkasten_body_start(content: &str) -> usize {
    // Body starts after the first "---" line
    if let Some(pos) = content.find("\n---\n") {
        return pos + 5; // skip "\n---\n"
    }
    if let Some(pos) = content.find("\n---") {
        if content[pos + 4..].starts_with('\n') || content.len() == pos + 4 {
            return pos + 4;
        }
    }
    0
}

fn try_parse_yaml_block(content: &str, _lines: &[&str]) -> Option<(Frontmatter, usize)> {
    if !content.starts_with("---\n") {
        return None;
    }

    let cap = YAML_BLOCK_RE.captures(content)?;
    let yaml_str = &cap[1];
    let yaml_end = cap.get(0).unwrap().end();

    // Try parsing YAML
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;

    let mut fm = Frontmatter::default();

    // Check if this looks like a Clippings file (has inline fields after YAML block)
    let after_yaml = &content[yaml_end..];
    let has_inline_fields = after_yaml.lines().any(|l| {
        let l = l.trim();
        !l.is_empty()
            && INLINE_FIELD_RE.is_match(l)
            && !l.starts_with('#')
    });

    if has_inline_fields {
        fm.format = Some(FrontmatterFormat::Clippings);
        // Parse inline fields after YAML block
        parse_clippings_inline(&mut fm, content, yaml_end);
    } else {
        fm.format = Some(FrontmatterFormat::StandardYaml);
    }

    // Extract common fields from YAML
    if let serde_yaml::Value::Mapping(map) = &yaml_value {
        // Don't use template titles from Clippings (usually unreliable)
        if fm.format != Some(FrontmatterFormat::Clippings) {
            if let Some(serde_yaml::Value::String(title)) = map.get("title") {
                fm.title = Some(title.clone());
            }
        }

        if let Some(serde_yaml::Value::Sequence(tags)) = map.get("tags") {
            for tag in tags {
                if let serde_yaml::Value::String(t) = tag {
                    let tag_str = if t.starts_with('#') {
                        t.clone()
                    } else {
                        format!("#{t}")
                    };
                    if !fm.tags.contains(&tag_str) {
                        fm.tags.push(tag_str);
                    }
                }
            }
        }
    }

    // Calculate body start: skip past YAML block + any blank lines
    let body_start = content[yaml_end..]
        .find(|c: char| c != '\n' && c != '\r')
        .map(|offset| yaml_end + offset)
        .unwrap_or(yaml_end);

    Some((fm, body_start))
}

fn parse_clippings_inline(fm: &mut Frontmatter, content: &str, yaml_end: usize) {
    let after_yaml = &content[yaml_end..];

    for line in after_yaml.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Stop at actual content (headings, paragraphs)
        if line.starts_with('#') && line.contains(' ') && !line.contains(':') {
            break;
        }

        if let Some(cap) = STATUS_RE.captures(line) {
            fm.status = Some(cap[1].trim().to_string());
            // Extract tags from status line too
            for tag_cap in TAG_IN_LINE_RE.captures_iter(&cap[1]) {
                let tag = format!("#{}", &tag_cap[1]);
                if !fm.tags.contains(&tag) {
                    fm.tags.push(tag);
                }
            }
        } else if let Some(cap) = TAGS_LINE_RE.captures(line) {
            for tag_cap in TAG_IN_LINE_RE.captures_iter(&cap[1]) {
                let tag = format!("#{}", &tag_cap[1]);
                if !fm.tags.contains(&tag) {
                    fm.tags.push(tag);
                }
            }
        } else if line.starts_with("author:") {
            let val = line.strip_prefix("author:").unwrap().trim();
            if val != "null" && !val.is_empty() {
                fm.author = Some(val.to_string());
            }
        } else if let Some(cap) = SOURCE_RE.captures(line) {
            fm.source = Some(cap[1].to_string());
        } else if line.starts_with("source:") {
            let val = line.strip_prefix("source:").unwrap().trim();
            if !val.is_empty() {
                fm.source = Some(val.to_string());
            }
        } else if line.starts_with("clipped:") {
            let val = line.strip_prefix("clipped:").unwrap().trim();
            if !val.is_empty() {
                fm.clipped = Some(val.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zettelkasten_format() {
        let content = "202212091326\nStatus: #idea\nTags: #imweb #devops\n\n---\nThis is the body";
        let (fm, body_start) = parse_frontmatter(content);
        assert_eq!(fm.format, Some(FrontmatterFormat::Zettelkasten));
        assert_eq!(fm.zettel_id.as_deref(), Some("202212091326"));
        assert_eq!(fm.status.as_deref(), Some("#idea"));
        assert!(fm.tags.contains(&"#imweb".to_string()));
        assert!(fm.tags.contains(&"#devops".to_string()));
        assert!(body_start > 0);
        assert!(content[body_start..].starts_with("This is the body"));
    }

    #[test]
    fn test_standard_yaml() {
        let content = "---\ntitle: My Note\ntags:\n  - rust\n  - cli\n---\nBody text here";
        let (fm, _body_start) = parse_frontmatter(content);
        assert_eq!(fm.format, Some(FrontmatterFormat::StandardYaml));
        assert_eq!(fm.title.as_deref(), Some("My Note"));
        assert!(fm.tags.contains(&"#rust".to_string()));
        assert!(fm.tags.contains(&"#cli".to_string()));
    }

    #[test]
    fn test_clippings_format() {
        let content = "---\ntitle: A New Hope\nyear: 1977\n---\nauthor: George Lucas\nsource: [link](https://example.com)\nclipped: 2023-12-18\nStatus: #clippings\nTags: #imweb #TDD\n";
        let (fm, _body_start) = parse_frontmatter(content);
        assert_eq!(fm.format, Some(FrontmatterFormat::Clippings));
        assert_eq!(fm.author.as_deref(), Some("George Lucas"));
        assert_eq!(fm.source.as_deref(), Some("https://example.com"));
        assert_eq!(fm.clipped.as_deref(), Some("2023-12-18"));
        assert!(fm.tags.contains(&"#clippings".to_string()));
        assert!(fm.tags.contains(&"#imweb".to_string()));
        assert!(fm.tags.contains(&"#TDD".to_string()));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "Just some regular markdown content";
        let (fm, body_start) = parse_frontmatter(content);
        assert_eq!(fm.format, Some(FrontmatterFormat::None));
        assert_eq!(body_start, 0);
    }
}
