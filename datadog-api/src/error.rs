use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Middleware error: {0}")]
    MiddlewareError(#[from] reqwest_middleware::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = Error::ConfigError("Invalid configuration".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Configuration error"));
        assert!(display.contains("Invalid configuration"));
    }

    #[test]
    fn test_api_error_display() {
        let error = Error::ApiError {
            status: 404,
            message: "Not found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("API error"));
        assert!(display.contains("404"));
        assert!(display.contains("Not found"));
    }

    #[test]
    fn test_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        let error = Error::JsonError(json_error.unwrap_err());
        let display = format!("{}", error);
        assert!(display.contains("JSON serialization"));
    }

    #[test]
    fn test_invalid_response_error() {
        let error = Error::InvalidResponse("Bad format".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Invalid response format"));
        assert!(display.contains("Bad format"));
    }

    #[test]
    fn test_error_debug() {
        let error = Error::ConfigError("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("ConfigError"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_api_error_with_different_status_codes() {
        let codes = vec![400, 401, 403, 404, 500, 502, 503];
        for code in codes {
            let error = Error::ApiError {
                status: code,
                message: format!("Error {}", code),
            };
            let display = format!("{}", error);
            assert!(display.contains(&code.to_string()));
        }
    }
}
