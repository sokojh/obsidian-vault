use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Matches prefix:value patterns in search queries
    static ref PREFIX_RE: Regex = Regex::new(r"(\w+):(\S+)").unwrap();
}

/// Parsed search query with prefix filters extracted
pub struct ParsedQuery {
    /// Free text for full-text search
    pub text: String,
    /// tag: filter values
    pub tags: Vec<String>,
    /// in: filter (directory)
    pub dirs: Vec<String>,
    /// title: filter
    pub titles: Vec<String>,
    /// date: filter
    pub dates: Vec<String>,
}

/// Parse a user query, extracting tag:, in:, title:, date: prefixes
pub fn parse_query(input: &str) -> ParsedQuery {
    let mut tags = Vec::new();
    let mut dirs = Vec::new();
    let mut titles = Vec::new();
    let mut dates = Vec::new();

    // Process each word
    let mut remaining = input.to_string();

    for cap in PREFIX_RE.captures_iter(input) {
        let full_match = cap.get(0).unwrap().as_str();
        let prefix = &cap[1];
        let value = &cap[2];

        match prefix {
            "tag" => {
                let tag = if value.starts_with('#') {
                    value.to_string()
                } else {
                    format!("#{value}")
                };
                tags.push(tag);
                remaining = remaining.replace(full_match, "");
            }
            "in" => {
                dirs.push(value.to_string());
                remaining = remaining.replace(full_match, "");
            }
            "title" => {
                titles.push(value.to_string());
                remaining = remaining.replace(full_match, "");
            }
            "date" => {
                dates.push(value.to_string());
                remaining = remaining.replace(full_match, "");
            }
            _ => {
                // Not a known prefix, keep as text
            }
        }
    }

    let text = remaining.trim().to_string();

    ParsedQuery {
        text,
        tags,
        dirs,
        titles,
        dates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let q = parse_query("kubernetes basics");
        assert_eq!(q.text, "kubernetes basics");
        assert!(q.tags.is_empty());
    }

    #[test]
    fn test_parse_tag_prefix() {
        let q = parse_query("tag:#devops deployment");
        assert_eq!(q.tags, vec!["#devops"]);
        assert_eq!(q.text.trim(), "deployment");
    }

    #[test]
    fn test_parse_multiple_prefixes() {
        let q = parse_query("tag:devops in:Zettelkasten kubernetes");
        assert_eq!(q.tags, vec!["#devops"]);
        assert_eq!(q.dirs, vec!["Zettelkasten"]);
        assert!(q.text.contains("kubernetes"));
    }

    #[test]
    fn test_parse_date_prefix() {
        let q = parse_query("date:2024-01 notes");
        assert_eq!(q.dates, vec!["2024-01"]);
        assert!(q.text.contains("notes"));
    }
}
