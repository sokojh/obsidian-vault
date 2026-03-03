use serde_json::Value;

/// Filter JSON output to only include specified fields
pub fn filter_fields(value: &Value, fields: &[String]) -> Value {
    match value {
        Value::Object(map) => {
            let filtered: serde_json::Map<String, Value> = map
                .iter()
                .filter(|(key, _)| fields.iter().any(|f| f == key.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Value::Object(filtered)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| filter_fields(v, fields)).collect())
        }
        _ => value.clone(),
    }
}

/// Parse comma-separated field list
pub fn parse_fields(fields_str: &str) -> Vec<String> {
    fields_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_fields() {
        let value = json!({"title": "Test", "path": "a.md", "tags": ["#a"]});
        let fields = vec!["title".to_string(), "tags".to_string()];
        let filtered = filter_fields(&value, &fields);
        assert!(filtered.get("title").is_some());
        assert!(filtered.get("tags").is_some());
        assert!(filtered.get("path").is_none());
    }

    #[test]
    fn test_filter_array() {
        let value = json!([
            {"title": "A", "path": "a.md"},
            {"title": "B", "path": "b.md"}
        ]);
        let fields = vec!["title".to_string()];
        let filtered = filter_fields(&value, &fields);
        let arr = filtered.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr[0].get("path").is_none());
    }
}
