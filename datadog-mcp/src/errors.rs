//! MCP error codes and helpers for consistent error responses

#![allow(dead_code)] // Helper functions defined for future use

use rmcp::model::{ErrorCode, ErrorData};

/// MCP-compliant error codes for Datadog MCP Server
#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum McpErrorCode {
    /// JSON-RPC parse error
    ParseError = -32700,
    /// Invalid JSON-RPC request
    InvalidRequest = -32600,
    /// Method not found
    MethodNotFound = -32601,
    /// Invalid method parameters
    InvalidParams = -32602,
    /// Internal server error
    InternalError = -32603,
    /// Configuration error (e.g., missing credentials)
    ConfigError = -32000,
    /// Datadog API returned an error
    ApiError = -32001,
    /// Network/connectivity issue
    NetworkError = -32002,
    /// Rate limit exceeded
    RateLimited = -32003,
    /// Resource not found
    NotFound = -32004,
    /// Authentication/authorization failure
    AuthError = -32005,
}

impl McpErrorCode {
    pub fn to_error_data(self, message: impl Into<String>) -> ErrorData {
        ErrorData::new(ErrorCode(self as i32), message.into(), None)
    }
}

/// Convert an anyhow error to appropriate MCP error data
pub fn to_mcp_error(err: anyhow::Error) -> ErrorData {
    let err_str = err.to_string();

    // Try to determine the error type from the message
    let code = if err_str.contains("DD_API_KEY") || err_str.contains("DD_APP_KEY") {
        McpErrorCode::ConfigError
    } else if err_str.contains("401") || err_str.contains("403") || err_str.contains("Forbidden") {
        McpErrorCode::AuthError
    } else if err_str.contains("404") || err_str.contains("not found") {
        McpErrorCode::NotFound
    } else if err_str.contains("429") || err_str.contains("rate limit") {
        McpErrorCode::RateLimited
    } else if err_str.contains("timeout")
        || err_str.contains("connection")
        || err_str.contains("network")
    {
        McpErrorCode::NetworkError
    } else if err_str.contains("status") || err_str.contains("API error") {
        McpErrorCode::ApiError
    } else {
        McpErrorCode::InternalError
    };

    code.to_error_data(err_str)
}

/// Create an API error with additional context
pub fn api_error(operation: &str, error: impl std::fmt::Display) -> ErrorData {
    McpErrorCode::ApiError.to_error_data(format!(
        "Failed to {}: {}. Check API credentials and permissions.",
        operation, error
    ))
}

/// Create a validation error for invalid parameters
pub fn validation_error(message: impl Into<String>) -> ErrorData {
    McpErrorCode::InvalidParams.to_error_data(message)
}

/// Create a not found error for missing resources
pub fn not_found_error(resource: &str, id: impl std::fmt::Display) -> ErrorData {
    McpErrorCode::NotFound.to_error_data(format!("{} with ID '{}' not found", resource, id))
}

/// Create a configuration error
pub fn config_error(message: impl Into<String>) -> ErrorData {
    McpErrorCode::ConfigError.to_error_data(message)
}

/// Create a rate limit error
pub fn rate_limited_error() -> ErrorData {
    McpErrorCode::RateLimited.to_error_data("Rate limit exceeded. Please wait and retry.")
}

/// Create an auth error
pub fn auth_error(message: impl Into<String>) -> ErrorData {
    McpErrorCode::AuthError.to_error_data(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(McpErrorCode::ParseError as i32, -32700);
        assert_eq!(McpErrorCode::InvalidRequest as i32, -32600);
        assert_eq!(McpErrorCode::MethodNotFound as i32, -32601);
        assert_eq!(McpErrorCode::InvalidParams as i32, -32602);
        assert_eq!(McpErrorCode::InternalError as i32, -32603);
        assert_eq!(McpErrorCode::ConfigError as i32, -32000);
        assert_eq!(McpErrorCode::ApiError as i32, -32001);
        assert_eq!(McpErrorCode::NetworkError as i32, -32002);
        assert_eq!(McpErrorCode::RateLimited as i32, -32003);
        assert_eq!(McpErrorCode::NotFound as i32, -32004);
        assert_eq!(McpErrorCode::AuthError as i32, -32005);
    }

    #[test]
    fn test_to_error_data() {
        let err = McpErrorCode::ApiError.to_error_data("test message");
        assert_eq!(err.code.0, -32001);
        assert_eq!(err.message, "test message");
    }

    #[test]
    fn test_to_mcp_error_config() {
        let err = anyhow::anyhow!("DD_API_KEY not set");
        let data = to_mcp_error(err);
        assert_eq!(data.code.0, McpErrorCode::ConfigError as i32);
    }

    #[test]
    fn test_to_mcp_error_auth() {
        let err = anyhow::anyhow!("API error: 403 Forbidden");
        let data = to_mcp_error(err);
        assert_eq!(data.code.0, McpErrorCode::AuthError as i32);
    }

    #[test]
    fn test_to_mcp_error_not_found() {
        let err = anyhow::anyhow!("Resource not found");
        let data = to_mcp_error(err);
        assert_eq!(data.code.0, McpErrorCode::NotFound as i32);
    }

    #[test]
    fn test_to_mcp_error_rate_limited() {
        let err = anyhow::anyhow!("429 rate limit exceeded");
        let data = to_mcp_error(err);
        assert_eq!(data.code.0, McpErrorCode::RateLimited as i32);
    }

    #[test]
    fn test_to_mcp_error_network() {
        let err = anyhow::anyhow!("connection timeout");
        let data = to_mcp_error(err);
        assert_eq!(data.code.0, McpErrorCode::NetworkError as i32);
    }

    #[test]
    fn test_helper_functions() {
        let err = api_error("get monitors", "connection failed");
        assert_eq!(err.code.0, McpErrorCode::ApiError as i32);
        assert!(err.message.contains("get monitors"));

        let err = validation_error("invalid monitor_id");
        assert_eq!(err.code.0, McpErrorCode::InvalidParams as i32);

        let err = not_found_error("Monitor", 123);
        assert_eq!(err.code.0, McpErrorCode::NotFound as i32);
        assert!(err.message.contains("123"));

        let err = config_error("missing API key");
        assert_eq!(err.code.0, McpErrorCode::ConfigError as i32);

        let err = rate_limited_error();
        assert_eq!(err.code.0, McpErrorCode::RateLimited as i32);

        let err = auth_error("invalid credentials");
        assert_eq!(err.code.0, McpErrorCode::AuthError as i32);
    }
}
