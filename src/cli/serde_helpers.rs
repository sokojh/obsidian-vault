use serde::de;

/// Deserialize a value that can be either a string or an array of strings.
/// Arrays are joined with commas: ["a","b"] → "a,b"
/// Used for `tags` and `sections` fields so agents can pass JSON arrays naturally.
pub fn string_or_array<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringOrArray;

    impl<'de> de::Visitor<'de> for StringOrArray {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string, an array of strings, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                Ok(None)
            } else {
                Ok(Some(v.to_string()))
            }
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            if v.is_empty() {
                Ok(None)
            } else {
                Ok(Some(v))
            }
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                items.push(item);
            }
            if items.is_empty() {
                Ok(None)
            } else {
                Ok(Some(items.join(",")))
            }
        }
    }

    deserializer.deserialize_any(StringOrArray)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Test {
        #[serde(default, deserialize_with = "string_or_array")]
        tags: Option<String>,
    }

    #[test]
    fn test_string_input() {
        let t: Test = serde_json::from_str(r#"{"tags":"a,b,c"}"#).unwrap();
        assert_eq!(t.tags, Some("a,b,c".to_string()));
    }

    #[test]
    fn test_array_input() {
        let t: Test = serde_json::from_str(r#"{"tags":["a","b","c"]}"#).unwrap();
        assert_eq!(t.tags, Some("a,b,c".to_string()));
    }

    #[test]
    fn test_null_input() {
        let t: Test = serde_json::from_str(r#"{"tags":null}"#).unwrap();
        assert_eq!(t.tags, None);
    }

    #[test]
    fn test_missing_input() {
        let t: Test = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(t.tags, None);
    }

    #[test]
    fn test_empty_array() {
        let t: Test = serde_json::from_str(r#"{"tags":[]}"#).unwrap();
        assert_eq!(t.tags, None);
    }

    #[test]
    fn test_empty_string() {
        let t: Test = serde_json::from_str(r#"{"tags":""}"#).unwrap();
        assert_eq!(t.tags, None);
    }
}
