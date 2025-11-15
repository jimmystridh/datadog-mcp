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
