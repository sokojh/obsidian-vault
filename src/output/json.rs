use std::collections::HashMap;

use serde::Serialize;

use crate::error::OvError;

/// Standard API response wrapper — all successful responses use this
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
}

/// Structured error response for agents
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ErrorResponse {
    pub fn from_error(e: &OvError) -> Self {
        Self {
            ok: false,
            error: ErrorDetail {
                code: e.error_code().to_string(),
                message: e.to_string(),
                hint: e.hint().map(|s| s.to_string()),
            },
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"ok":false,"error":{"code":"SERIALIZE_ERROR","message":"Failed to serialize error"}}"#.to_string()
        })
    }
}
