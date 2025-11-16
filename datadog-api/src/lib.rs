//! # Datadog API Client Library
//!
//! A Rust client library for interacting with the Datadog API.
//!
//! This library provides a type-safe, async interface to Datadog's monitoring and observability
//! platform. It handles authentication, request building, and response parsing for all major
//! Datadog API endpoints.
//!
//! ## Features
//!
//! - **Async/await support** - Built on tokio for non-blocking I/O
//! - **Type-safe** - Comprehensive type definitions for all API resources
//! - **Modular** - Organized by API category (metrics, monitors, dashboards, etc.)
//! - **Error handling** - Detailed error types for API failures
//! - **Configuration** - Support for multiple Datadog regions
//!
//! ## Quick Start
//!
//! ```no_run
//! use datadog_api::{DatadogClient, DatadogConfig};
//! use datadog_api::apis::MetricsApi;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment variables
//!     let config = DatadogConfig::from_env()?;
//!
//!     // Create a client
//!     let client = DatadogClient::new(config)?;
//!
//!     // Use an API
//!     let metrics_api = MetricsApi::new(client);
//!     let metrics = metrics_api.list_metrics("system.cpu").await?;
//!
//!     println!("Found {} metrics", metrics.metrics.unwrap_or_default().len());
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! The library requires Datadog API credentials which can be provided via:
//!
//! 1. Environment variables (recommended):
//!    - `DD_API_KEY` - Your Datadog API key
//!    - `DD_APP_KEY` - Your Datadog application key
//!    - `DD_SITE` - Optional Datadog site (defaults to datadoghq.com)
//!
//! 2. Programmatically:
//!    ```no_run
//!    use datadog_api::DatadogConfig;
//!
//!    let config = DatadogConfig::new(
//!        "api_key".to_string(),
//!        "app_key".to_string()
//!    ).with_site("datadoghq.eu".to_string());
//!    ```
//!
//! ## Supported Datadog Sites
//!
//! - US1: `datadoghq.com` (default)
//! - US3: `us3.datadoghq.com`
//! - US5: `us5.datadoghq.com`
//! - EU: `datadoghq.eu`
//! - AP1: `ap1.datadoghq.com`
//! - US1-FED: `ddog-gov.com`

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod apis;

pub use client::DatadogClient;
pub use config::DatadogConfig;
pub use error::{Error, Result};

