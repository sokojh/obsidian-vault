use std::collections::HashMap;

use serde::Serialize;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub ok: bool,
    pub count: usize,
    pub data: T,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, serde_json::Value>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T, count: usize) -> Self {
        Self {
            ok: true,
            count,
            data,
            meta: HashMap::new(),
        }
    }

    pub fn with_meta(mut self, key: &str, value: serde_json::Value) -> Self {
        self.meta.insert(key.to_string(), value);
        self
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: String,
    pub code: i32,
}

impl ErrorResponse {
    pub fn new(error: &str, code: i32) -> Self {
        Self {
            ok: false,
            error: error.to_string(),
            code,
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
