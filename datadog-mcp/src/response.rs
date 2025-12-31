//! Tool response helpers for consistent MCP responses
//!
//! Provides helper functions and macros to reduce duplication in tool implementations.

use crate::cache::store_data;
use crate::output::{Formattable, OutputFormat};
use serde::Serialize;
use serde_json::{json, Value};
use tracing::{error, info, warn};

/// Maximum response size in bytes before warning (1 MB)
pub const RESPONSE_SIZE_WARN_THRESHOLD: usize = 1024 * 1024;

/// Maximum response size in bytes before truncation (10 MB)
pub const RESPONSE_SIZE_MAX: usize = 10 * 1024 * 1024;

/// Check response size and log warnings if needed
fn check_response_size<T: Serialize>(data: &T, prefix: &str) -> Option<usize> {
    match serde_json::to_string(data) {
        Ok(json_str) => {
            let size = json_str.len();
            if size > RESPONSE_SIZE_MAX {
                warn!(
                    "{}: Response size ({} bytes) exceeds maximum ({} bytes), data may be truncated",
                    prefix, size, RESPONSE_SIZE_MAX
                );
            } else if size > RESPONSE_SIZE_WARN_THRESHOLD {
                warn!(
                    "{}: Large response size ({} bytes), consider using pagination",
                    prefix, size
                );
            }
            Some(size)
        }
        Err(_) => None,
    }
}

/// Create a successful tool response with cached data
pub async fn tool_success<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
    summary: impl Into<String>,
) -> anyhow::Result<Value> {
    check_response_size(data, prefix);
    let filepath = store_data(data, prefix, format).await?;
    let summary = summary.into();
    info!("{}", summary);
    Ok(json!({
        "filepath": filepath,
        "summary": summary,
        "status": "success",
    }))
}

/// Create a successful tool response with cached data and additional fields
pub async fn tool_success_with_fields<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
    summary: impl Into<String>,
    additional_fields: Value,
) -> anyhow::Result<Value> {
    check_response_size(data, prefix);
    let filepath = store_data(data, prefix, format).await?;
    let summary = summary.into();
    info!("{}", summary);

    let mut response = json!({
        "filepath": filepath,
        "summary": summary,
        "status": "success",
    });

    // Merge additional fields
    if let (Some(base), Some(extra)) = (response.as_object_mut(), additional_fields.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }

    Ok(response)
}

/// Create an error tool response
pub fn tool_error(operation: &str, err: impl std::fmt::Display) -> Value {
    let msg = format!("{}: {}", operation, err);
    error!("{}", msg);
    json!({
        "error": msg,
        "status": "error",
    })
}

/// Create a simple success response without caching (for operations like delete)
pub fn simple_success(summary: impl Into<String>) -> Value {
    let summary = summary.into();
    info!("{}", summary);
    json!({
        "summary": summary,
        "status": "success",
    })
}

/// Create a simple success response with additional fields
pub fn simple_success_with_fields(summary: impl Into<String>, additional_fields: Value) -> Value {
    let summary = summary.into();
    info!("{}", summary);

    let mut response = json!({
        "summary": summary,
        "status": "success",
    });

    if let (Some(base), Some(extra)) = (response.as_object_mut(), additional_fields.as_object()) {
        for (key, value) in extra {
            base.insert(key.clone(), value.clone());
        }
    }

    response
}

/// Macro for the common tool response pattern
///
/// # Usage
/// ```ignore
/// tool_response!(result, "metrics", ctx, "Retrieved metrics data")
/// ```
#[macro_export]
macro_rules! tool_response {
    ($result:expr, $prefix:expr, $ctx:expr, $summary:expr) => {
        match $result {
            Ok(data) => {
                $crate::response::tool_success(&data, $prefix, $ctx.output_format, $summary).await
            }
            Err(e) => Ok($crate::response::tool_error($prefix, e)),
        }
    };
}

/// Macro for tool response with additional fields
///
/// # Usage
/// ```ignore
/// tool_response_with_fields!(result, "monitors", ctx, summary, json!({"count": 5}))
/// ```
#[macro_export]
macro_rules! tool_response_with_fields {
    ($result:expr, $prefix:expr, $ctx:expr, $data_ident:ident, $summary:expr, $fields:block) => {
        match $result {
            Ok($data_ident) => {
                $crate::response::tool_success_with_fields(
                    &$data_ident,
                    $prefix,
                    $ctx.output_format,
                    $summary,
                    $fields,
                )
                .await
            }
            Err(e) => Ok($crate::response::tool_error($prefix, e)),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error() {
        let result = tool_error("get_monitors", "connection failed");
        assert_eq!(result["status"], "error");
        assert!(result["error"].as_str().unwrap().contains("get_monitors"));
        assert!(result["error"]
            .as_str()
            .unwrap()
            .contains("connection failed"));
    }

    #[test]
    fn test_simple_success() {
        let result = simple_success("Operation completed");
        assert_eq!(result["status"], "success");
        assert_eq!(result["summary"], "Operation completed");
    }

    #[test]
    fn test_simple_success_with_fields() {
        let result = simple_success_with_fields("Deleted monitor", json!({"monitor_id": 123}));
        assert_eq!(result["status"], "success");
        assert_eq!(result["summary"], "Deleted monitor");
        assert_eq!(result["monitor_id"], 123);
    }

    #[test]
    fn test_check_response_size_small() {
        let data = json!({"key": "value"});
        let size = check_response_size(&data, "test");
        assert!(size.is_some());
        assert!(size.unwrap() < RESPONSE_SIZE_WARN_THRESHOLD);
    }

    #[test]
    fn test_check_response_size_returns_correct_size() {
        let data = json!({"test": "data"});
        let size = check_response_size(&data, "test").unwrap();
        let expected_size = serde_json::to_string(&data).unwrap().len();
        assert_eq!(size, expected_size);
    }

    #[test]
    fn test_response_size_thresholds() {
        assert!(RESPONSE_SIZE_WARN_THRESHOLD < RESPONSE_SIZE_MAX);
        assert_eq!(RESPONSE_SIZE_WARN_THRESHOLD, 1024 * 1024);
        assert_eq!(RESPONSE_SIZE_MAX, 10 * 1024 * 1024);
    }
}
