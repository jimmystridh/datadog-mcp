pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod apis;

pub use client::DatadogClient;
pub use config::DatadogConfig;
pub use error::{Error, Result};
