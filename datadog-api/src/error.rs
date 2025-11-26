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

impl Error {
    /// Returns true if this is a client error (4xx status code)
    #[must_use]
    pub fn is_client_error(&self) -> bool {
        matches!(self, Error::ApiError { status, .. } if (400..500).contains(status))
    }

    /// Returns true if this is a server error (5xx status code)
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        matches!(self, Error::ApiError { status, .. } if (500..600).contains(status))
    }

    /// Returns true if this is a not found error (404)
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::ApiError { status: 404, .. })
    }

    /// Returns true if this is an authentication error (401)
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Error::ApiError { status: 401, .. })
    }

    /// Returns true if this is a forbidden error (403)
    #[must_use]
    pub fn is_forbidden(&self) -> bool {
        matches!(self, Error::ApiError { status: 403, .. })
    }

    /// Returns true if this is a rate limit error (429)
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Error::ApiError { status: 429, .. })
    }

    /// Returns the HTTP status code if this is an API error
    #[must_use]
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Error::ApiError { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Returns true if this error is likely transient and retrying might succeed
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::ApiError { status, .. } => {
                // 429 (rate limit), 500, 502, 503, 504 are retryable
                *status == 429 || (500..=504).contains(status)
            }
            Error::HttpError(e) => e.is_connect() || e.is_timeout(),
            Error::MiddlewareError(_) => true,
            _ => false,
        }
    }
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

    #[test]
    fn test_is_client_error() {
        let error_400 = Error::ApiError {
            status: 400,
            message: "Bad Request".into(),
        };
        let error_404 = Error::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        let error_500 = Error::ApiError {
            status: 500,
            message: "Server Error".into(),
        };

        assert!(error_400.is_client_error());
        assert!(error_404.is_client_error());
        assert!(!error_500.is_client_error());
    }

    #[test]
    fn test_is_server_error() {
        let error_400 = Error::ApiError {
            status: 400,
            message: "Bad Request".into(),
        };
        let error_500 = Error::ApiError {
            status: 500,
            message: "Server Error".into(),
        };
        let error_503 = Error::ApiError {
            status: 503,
            message: "Service Unavailable".into(),
        };

        assert!(!error_400.is_server_error());
        assert!(error_500.is_server_error());
        assert!(error_503.is_server_error());
    }

    #[test]
    fn test_specific_error_checks() {
        let not_found = Error::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        let unauthorized = Error::ApiError {
            status: 401,
            message: "Unauthorized".into(),
        };
        let forbidden = Error::ApiError {
            status: 403,
            message: "Forbidden".into(),
        };
        let rate_limited = Error::ApiError {
            status: 429,
            message: "Too Many Requests".into(),
        };

        assert!(not_found.is_not_found());
        assert!(unauthorized.is_unauthorized());
        assert!(forbidden.is_forbidden());
        assert!(rate_limited.is_rate_limited());

        assert!(!not_found.is_unauthorized());
        assert!(!unauthorized.is_not_found());
    }

    #[test]
    fn test_status_code() {
        let api_error = Error::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        let config_error = Error::ConfigError("test".into());

        assert_eq!(api_error.status_code(), Some(404));
        assert_eq!(config_error.status_code(), None);
    }

    #[test]
    fn test_is_retryable() {
        let rate_limited = Error::ApiError {
            status: 429,
            message: "Rate Limited".into(),
        };
        let server_error = Error::ApiError {
            status: 500,
            message: "Server Error".into(),
        };
        let bad_gateway = Error::ApiError {
            status: 502,
            message: "Bad Gateway".into(),
        };
        let not_found = Error::ApiError {
            status: 404,
            message: "Not Found".into(),
        };
        let bad_request = Error::ApiError {
            status: 400,
            message: "Bad Request".into(),
        };

        assert!(rate_limited.is_retryable());
        assert!(server_error.is_retryable());
        assert!(bad_gateway.is_retryable());
        assert!(!not_found.is_retryable());
        assert!(!bad_request.is_retryable());
    }
}
