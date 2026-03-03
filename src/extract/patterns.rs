use lazy_static::lazy_static;
use regex::Regex;

use crate::model::link::WikiLink;

lazy_static! {
    /// Matches [[target]] or [[target|alias]]
    static ref WIKILINK_RE: Regex =
        Regex::new(r"\[\[([^\]\|]+?)(?:\|([^\]]+?))?\]\]").unwrap();

    /// Matches ![[embed]]
    static ref EMBED_RE: Regex =
        Regex::new(r"!\[\[([^\]\|]+?)(?:\|([^\]]+?))?\]\]").unwrap();

    /// Matches #tag (not inside code blocks or URLs)
    /// Tags: alphanumeric, Korean, /, -, _
    static ref TAG_RE: Regex =
        Regex::new(r"(?:^|[\s,;(])#([\w\p{Hangul}/\-]+)").unwrap();

    /// Matches markdown headings: # Heading
    static ref HEADING_RE: Regex =
        Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
}

/// Extract all wiki links from text, with line numbers
pub fn extract_links(content: &str) -> Vec<WikiLink> {
    let mut links = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip code blocks
        if line.trim_start().starts_with("```") {
            continue;
        }

        // Extract embeds first (they contain [[]])
        for cap in EMBED_RE.captures_iter(line) {
            links.push(WikiLink {
                target: cap[1].trim().to_string(),
                alias: cap.get(2).map(|m| m.as_str().trim().to_string()),
                is_embed: true,
                line: line_num + 1,
            });
        }

        // Extract regular wiki links (exclude embeds by checking preceding char)
        // We need to avoid double-counting embeds
        let line_without_embeds = EMBED_RE.replace_all(line, "");
        for cap in WIKILINK_RE.captures_iter(&line_without_embeds) {
            links.push(WikiLink {
                target: cap[1].trim().to_string(),
                alias: cap.get(2).map(|m| m.as_str().trim().to_string()),
                is_embed: false,
                line: line_num + 1,
            });
        }
    }

    links
}

/// Extract all tags from text (both frontmatter and inline)
pub fn extract_inline_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        for cap in TAG_RE.captures_iter(line) {
            let tag = format!("#{}", &cap[1]);
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
    }

    tags
}

/// Extract headings from markdown content
pub fn extract_headings(content: &str) -> Vec<(u8, String)> {
    let mut headings = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if let Some(cap) = HEADING_RE.captures(line) {
            let level = cap[1].len() as u8;
            let text = cap[2].trim().to_string();
            headings.push((level, text));
        }
    }

    headings
}

/// Count words in text (approximate, handles mixed CJK/Latin)
pub fn word_count(content: &str) -> usize {
    content.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let content = "See [[Note One]] and [[Note Two|display text]]\n![[image.png]]";
        let links = extract_links(content);
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].target, "Note One");
        assert!(!links[0].is_embed);
        assert_eq!(links[1].target, "Note Two");
        assert_eq!(links[1].alias.as_deref(), Some("display text"));
        assert_eq!(links[2].target, "image.png");
        assert!(links[2].is_embed);
    }

    #[test]
    fn test_extract_inline_tags() {
        let content = "Some text #tag1 and #tag2/nested\n#한국어태그";
        let tags = extract_inline_tags(content);
        assert!(tags.contains(&"#tag1".to_string()));
        assert!(tags.contains(&"#tag2/nested".to_string()));
        assert!(tags.contains(&"#한국어태그".to_string()));
    }

    #[test]
    fn test_extract_tags_skip_code() {
        let content = "```\n#not_a_tag\n```\n#real_tag";
        let tags = extract_inline_tags(content);
        assert_eq!(tags, vec!["#real_tag"]);
    }

    #[test]
    fn test_extract_headings() {
        let content = "# Title\n## Section\ntext\n### Sub";
        let headings = extract_headings(content);
        assert_eq!(headings.len(), 3);
        assert_eq!(headings[0], (1, "Title".to_string()));
        assert_eq!(headings[1], (2, "Section".to_string()));
        assert_eq!(headings[2], (3, "Sub".to_string()));
    }
}
