pub mod fields;
pub mod human;
pub mod json;

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::error::OvError;

use self::json::{ApiResponse, ErrorResponse};

/// Print structured data in the requested format
pub fn print_output<T: Serialize>(
    data: T,
    count: usize,
    format: &OutputFormat,
    field_list: &Option<String>,
) {
    match format {
        OutputFormat::Json => {
            let response = ApiResponse::success(&data, count);
            let json_val = serde_json::to_value(&response).unwrap_or_default();

            let output = if let Some(fields_str) = field_list {
                let field_names = fields::parse_fields(fields_str);
                let mut filtered = json_val;
                if let Some(data) = filtered.get_mut("data") {
                    *data = fields::filter_fields(data, &field_names);
                }
                serde_json::to_string_pretty(&filtered).unwrap_or_default()
            } else {
                serde_json::to_string_pretty(&json_val).unwrap_or_default()
            };

            println!("{output}");
        }
        OutputFormat::Jsonl => {
            // For JSONL, serialize each item in data array on its own line
            let json_val = serde_json::to_value(&data).unwrap_or_default();
            if let Some(arr) = json_val.as_array() {
                for item in arr {
                    let line = if let Some(fields_str) = field_list {
                        let field_names = fields::parse_fields(fields_str);
                        let filtered = fields::filter_fields(item, &field_names);
                        serde_json::to_string(&filtered).unwrap_or_default()
                    } else {
                        serde_json::to_string(item).unwrap_or_default()
                    };
                    println!("{line}");
                }
            } else {
                println!("{}", serde_json::to_string(&json_val).unwrap_or_default());
            }
        }
        OutputFormat::Human => {
            // Human output is handled by each command directly
            // This is a fallback - just print JSON
            let response = ApiResponse::success(&data, count);
            println!("{}", response.to_json_string());
        }
    }
}

/// Print error in the requested format
pub fn print_error(error: &OvError, format: &OutputFormat) {
    match format {
        OutputFormat::Json | OutputFormat::Jsonl => {
            let response = ErrorResponse::new(&error.to_string(), error.exit_code());
            eprintln!("{}", response.to_json_string());
        }
        OutputFormat::Human => {
            eprintln!("error: {error}");
        }
    }
}
