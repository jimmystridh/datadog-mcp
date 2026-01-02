//! # Datadog MCP Server
//!
//! A Model Context Protocol (MCP) server exposing Datadog tools to AI assistants.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     datadog-mcp                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  server.rs          │  MCP server with tool registration   │
//! │  └─ DatadogMcpServer│  #[tool] macro handlers              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  state.rs           │  Shared server state                 │
//! │  ├─ ServerState     │  Config, client, output format       │
//! │  └─ ToolContext     │  Per-request context for tools       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  tools/             │  Domain-specific tool implementations│
//! │  ├─ monitors        │  Monitor CRUD                        │
//! │  ├─ dashboards      │  Dashboard management                │
//! │  ├─ metrics         │  Metrics queries                     │
//! │  ├─ logs            │  Log search                          │
//! │  ├─ synthetics      │  Synthetic tests                     │
//! │  ├─ downtimes       │  Scheduled downtimes                 │
//! │  └─ ...             │  Events, infrastructure, etc.        │
//! ├─────────────────────────────────────────────────────────────┤
//! │  tool_inputs.rs     │  Typed input schemas (schemars)      │
//! │  ids.rs             │  Type-safe ID newtypes               │
//! │  input_validation   │  Monitor/dashboard validation        │
//! │  sanitize.rs        │  Input sanitization                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │  response.rs        │  Tool response helpers & macros      │
//! │  output.rs          │  JSON/TOON output formatting         │
//! │  cache.rs           │  Response caching to files           │
//! │  errors.rs          │  MCP error code mapping              │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Request Flow
//!
//! ```text
//! MCP Client (Claude, etc.)
//!     │
//!     ▼ JSON-RPC over stdio
//! ┌───────────────────┐
//! │  DatadogMcpServer │  Parse request, dispatch to tool
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │    Tool Handler   │  Validate inputs, sanitize
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │   datadog-api     │  Make HTTP request to Datadog
//! └─────────┬─────────┘
//!           │
//!           ▼
//! ┌───────────────────┐
//! │  Response Handler │  Format (JSON/TOON), cache to file
//! └───────────────────┘
//! ```
//!
//! ## Cargo Features
//!
//! - `toon` (default): TOON format for 30-60% token savings
//! - `keyring` (default): Secure credential storage
//!
//! Build minimal: `cargo build --no-default-features`

// Library exports for testing and potential library use
pub mod cache;
pub mod errors;
pub mod ids;
pub mod input_validation;
pub mod output;
#[macro_use]
pub mod response;
pub mod sanitize;
pub mod server;
pub mod state;
pub mod tool_inputs;
pub mod tools;
