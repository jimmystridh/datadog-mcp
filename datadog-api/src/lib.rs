//! # Datadog API Client Library
//!
//! A Rust client for the Datadog API with type-safe access to monitors,
//! dashboards, metrics, logs, synthetics, and more.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      datadog-api                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  config.rs          │  Configuration & credentials         │
//! │  ├─ DatadogConfig   │  API keys, site, retry settings      │
//! │  └─ SecretString    │  Zeroize-on-drop credential wrapper  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  client.rs          │  HTTP client with middleware         │
//! │  └─ DatadogClient   │  Retry logic, auth headers, gzip     │
//! ├─────────────────────────────────────────────────────────────┤
//! │  apis/              │  Domain-specific API modules         │
//! │  ├─ monitors        │  Monitor CRUD operations             │
//! │  ├─ dashboards      │  Dashboard management                │
//! │  ├─ metrics         │  Metrics queries                     │
//! │  ├─ logs            │  Log search                          │
//! │  ├─ synthetics      │  Synthetic tests                     │
//! │  ├─ events          │  Event stream                        │
//! │  ├─ infrastructure  │  Hosts and tags                      │
//! │  ├─ downtimes       │  Scheduled downtimes                 │
//! │  ├─ incidents       │  Incident management                 │
//! │  ├─ slos            │  Service Level Objectives            │
//! │  ├─ security        │  Security rules                      │
//! │  ├─ notebooks       │  Notebooks                           │
//! │  ├─ teams/users     │  Team and user management            │
//! │  └─ traces          │  APM traces                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │  models/            │  Request/response types (Serde)      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  error.rs           │  Error types with helper methods     │
//! │  └─ Error           │  is_retryable, is_not_found, etc.    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```no_run
//! use datadog_api::{DatadogClient, DatadogConfig};
//! use datadog_api::apis::MetricsApi;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DatadogConfig::from_env()?;
//!     let client = DatadogClient::new(config)?;
//!     let metrics_api = MetricsApi::new(client);
//!     let metrics = metrics_api.list_metrics("system.cpu").await?;
//!     println!("Found {} metrics", metrics.metrics.unwrap_or_default().len());
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration Sources
//!
//! Credentials load from (in order):
//! 1. **File**: `~/.datadog-mcp/credentials.json`
//! 2. **Keyring**: System credential storage (requires `keyring` feature)
//! 3. **Environment**: `DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`
//!
//! Use `DatadogConfig::from_env_or_file()` to try all sources.
//!
//! ## Error Handling
//!
//! ```no_run
//! # use datadog_api::Error;
//! fn handle(e: &Error) {
//!     if e.is_not_found() { /* 404 */ }
//!     else if e.is_rate_limited() { /* 429 - back off */ }
//!     else if e.is_retryable() { /* transient - retry */ }
//! }
//! ```
//!
//! ## Supported Sites
//!
//! - US1: `datadoghq.com` (default)
//! - US3: `us3.datadoghq.com`
//! - US5: `us5.datadoghq.com`
//! - EU: `datadoghq.eu`
//! - AP1: `ap1.datadoghq.com`
//! - US1-FED: `ddog-gov.com`
//!
//! ## Cargo Features
//!
//! - `keyring` (default): Secure credential storage in system keyring

pub mod apis;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod pagination;
pub mod rate_limit;
pub mod timestamp;

pub use client::{CacheInfo, CachedResponse, DatadogClient};
pub use config::{DatadogConfig, HttpConfig, RetryConfig};
pub use error::{Error, Result};
pub use models::{
    GroupDefinition, HeatmapDefinition, NoteDefinition, QueryTableDefinition,
    QueryValueDefinition, TemplateVariable, TimeseriesDefinition, ToplistDefinition, Widget,
    WidgetDefinition, WidgetLayout,
};
pub use pagination::{CursorParams, PageParams, PaginatedResponse, PaginationMeta};
pub use rate_limit::{RateLimitConfig, RateLimiter};
pub use timestamp::{TimestampMillis, TimestampNanos, TimestampSecs};
