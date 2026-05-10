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

    /// Attach `next_actions: [{cmd, why}, ...]` so agents see suggested follow-up commands.
    /// Reserved typed-builder API; current call sites use raw `with_meta("next_actions", json!(...))`.
    #[allow(dead_code)]
    pub fn with_next_actions(self, actions: Vec<NextAction>) -> Self {
        if actions.is_empty() {
            return self;
        }
        self.with_meta(
            "next_actions",
            serde_json::to_value(actions).unwrap_or_default(),
        )
    }
}

/// Suggested follow-up command for agents. Embedded in successful responses' `meta.next_actions`.
/// Currently constructed via the `serde_json::json!` macro; this type is reserved for a future
/// typed builder API (see `ApiResponse::with_next_actions`).
#[allow(dead_code)]
#[derive(Debug, Serialize, Clone)]
pub struct NextAction {
    /// Shell command an agent could run next, e.g. "ov index build"
    pub cmd: String,
    /// Why this is suggested, in one short sentence
    pub why: String,
}

#[allow(dead_code)]
impl NextAction {
    pub fn new(cmd: &str, why: &str) -> Self {
        Self {
            cmd: cmd.to_string(),
            why: why.to_string(),
        }
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
