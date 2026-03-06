pub mod fields;
pub mod json;

use serde::Serialize;

use self::json::ApiResponse;

/// Print structured data as JSON. Supports field filtering and JSONL streaming.
pub fn print_output<T: Serialize>(
    data: T,
    count: usize,
    jsonl: bool,
    field_list: &Option<String>,
) {
    if jsonl {
        print_jsonl(&data, field_list);
    } else {
        print_json(data, count, field_list);
    }
}

fn print_json<T: Serialize>(data: T, count: usize, field_list: &Option<String>) {
    let response = ApiResponse::success(&data, count);
    let json_val = serde_json::to_value(&response).unwrap_or_default();

    let output = if let Some(fields_str) = field_list {
        let field_names = fields::parse_fields(fields_str);
        let mut filtered = json_val;
        if let Some(data) = filtered.get_mut("data") {
            *data = fields::filter_fields(data, &field_names);
        }
        serde_json::to_string(&filtered).unwrap_or_default()
    } else {
        serde_json::to_string(&json_val).unwrap_or_default()
    };

    println!("{output}");
}

fn print_jsonl<T: Serialize>(data: &T, field_list: &Option<String>) {
    let json_val = serde_json::to_value(data).unwrap_or_default();
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
        println!(
            "{}",
            serde_json::to_string(&json_val).unwrap_or_default()
        );
    }
}

/// Print a successful response with metadata (pagination, etc.)
pub fn print_with_meta<T: Serialize>(
    data: T,
    count: usize,
    jsonl: bool,
    field_list: &Option<String>,
    meta: Vec<(&str, serde_json::Value)>,
) {
    if jsonl {
        print_jsonl(&data, field_list);
    } else {
        let mut response = ApiResponse::success(&data, count);
        for (key, value) in meta {
            response = response.with_meta(key, value);
        }
        let json_val = serde_json::to_value(&response).unwrap_or_default();

        let output = if let Some(fields_str) = field_list {
            let field_names = fields::parse_fields(fields_str);
            let mut filtered = json_val;
            if let Some(data) = filtered.get_mut("data") {
                *data = fields::filter_fields(data, &field_names);
            }
            serde_json::to_string(&filtered).unwrap_or_default()
        } else {
            serde_json::to_string(&json_val).unwrap_or_default()
        };

        println!("{output}");
    }
}
