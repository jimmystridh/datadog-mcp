# Datadog MCP Server (Rust Implementation)

A Model Context Protocol (MCP) server that provides programmatic access to Datadog's monitoring and observability platform through an AI-friendly interface, implemented in Rust.

## Features

- **31 MCP Tools** organized into 8 categories:
  - Metrics & Monitoring (9 tools)
  - Dashboards & Visualization (5 tools)
  - Logs & Events (2 tools)
  - Infrastructure & Tags (5 tools)
  - Testing & Applications (1 tool)
  - Security & Incidents (4 tools)
  - Teams & Users (2 tools)
  - Utilities (2 tools)

- **Async/Non-blocking Architecture** using Tokio
- **Automatic Retry Logic** with exponential backoff for rate limiting
- **Local JSON Caching** of all API responses
- **Modular Design** with separate API client and MCP server crates
- **Type-Safe** implementation using Rust's type system

## Architecture

The project is structured as a Rust workspace with two crates:

1. **datadog-api** - A standalone Datadog API client library
   - HTTP client with retry middleware
   - Authentication and configuration management
   - Type-safe API models and endpoints
   - Can be used independently in other projects

2. **datadog-mcp** - The MCP server implementation
   - All 31 MCP tools
   - Caching mechanism
   - Tool registration and routing

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Datadog API and Application keys
- MCP-compatible client (VS Code, Claude Desktop, Cursor, etc.)

## Installation

### 1. Clone the repository

```bash
git clone <repository-url>
cd claude_jungle_anaconda
```

### 2. Configure environment variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` and add your Datadog credentials:

```env
DD_API_KEY=your_api_key_here
DD_APP_KEY=your_app_key_here
DD_SITE=datadoghq.com  # or datadoghq.eu, us3.datadoghq.com, etc.
```

### 3. Build the project

```bash
cargo build --release
```

The binary will be available at `target/release/datadog-mcp`

## Usage

### Running the server

The MCP server communicates via stdio:

```bash
./target/release/datadog-mcp
```

### MCP Client Configuration

#### Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/path/to/datadog-mcp/target/release/datadog-mcp",
      "env": {
        "DD_API_KEY": "your_api_key",
        "DD_APP_KEY": "your_app_key",
        "DD_SITE": "datadoghq.com"
      }
    }
  }
}
```

#### VS Code with Continue

Add to your Continue configuration:

```json
{
  "mcpServers": [
    {
      "name": "datadog",
      "command": "/path/to/datadog-mcp/target/release/datadog-mcp",
      "env": {
        "DD_API_KEY": "your_api_key",
        "DD_APP_KEY": "your_app_key",
        "DD_SITE": "datadoghq.com"
      }
    }
  ]
}
```

## Available Tools

### Metrics & Monitoring

1. **validate_api_key** - Test API credentials
2. **get_metrics** - Query time series data
3. **search_metrics** - Find metrics by pattern
4. **get_metric_metadata** - Retrieve metric metadata
5. **get_monitors** - List monitoring alerts
6. **get_monitor** - Fetch specific monitor details
7. **create_monitor** - Create new monitoring alerts
8. **update_monitor** - Modify existing monitors
9. **delete_monitor** - Remove monitors

### Dashboards & Visualization

10. **get_dashboards** - List all dashboards
11. **get_dashboard** - Retrieve dashboard details
12. **create_dashboard** - Build new dashboards
13. **update_dashboard** - Modify existing dashboards
14. **delete_dashboard** - Remove dashboards

### Logs & Events

15. **search_logs** - Query log entries
16. **get_events** - Retrieve system events

### Infrastructure & Tags

17. **get_infrastructure** - Obtain host information
18. **get_tags** - Obtain host tags
19. **get_downtimes** - List scheduled downtimes
20. **create_downtime** - Schedule maintenance windows

### Testing & Applications

21. **get_synthetics_tests** - Retrieve synthetic tests

### Security & Incidents

22. **get_security_rules** - Retrieve security monitoring rules
23. **get_incidents** - Access incident data (with pagination)
24. **get_slos** - Obtain Service Level Objectives
25. **get_notebooks** - Retrieve Datadog notebooks

### Teams & Users

26. **get_teams** - Access team information
27. **get_users** - Retrieve user data

### Utilities

28. **analyze_data** - Analyze cached data (summary, stats, or trends)
29. **cleanup_cache** - Remove old cache files

## Caching

All API responses are automatically cached to the `datadog_cache/` directory in JSON format. Cache files are named using the pattern:

```
{prefix}_{timestamp}_{uuid}.json
```

Use the `cleanup_cache` tool to remove old cache files:

```
cleanup_cache(older_than_hours: 24)
```

## Development

### Project Structure

```
claude_jungle_anaconda/
├── Cargo.toml              # Workspace configuration
├── datadog-api/            # API client library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── client.rs       # HTTP client with retry logic
│       ├── config.rs       # Configuration management
│       ├── error.rs        # Error types
│       ├── models.rs       # API data models
│       └── apis/           # API endpoint modules
│           ├── metrics.rs
│           ├── monitors.rs
│           └── ...
└── datadog-mcp/            # MCP server
    ├── Cargo.toml
    └── src/
        ├── main.rs         # Server entry point
        ├── cache.rs        # Caching implementation
        ├── tools.rs        # Tool implementations (part 1)
        └── tools_part2.rs  # Tool implementations (part 2)
```

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running with Debug Logging

```bash
RUST_LOG=debug ./target/debug/datadog-mcp
```

## Using the Datadog API Client Independently

The `datadog-api` crate can be used as a standalone library:

```rust
use datadog_api::{DatadogClient, DatadogConfig, apis::MetricsApi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;

    let metrics_api = MetricsApi::new(client);
    let result = metrics_api.list_metrics("system.cpu").await?;

    println!("{:?}", result);
    Ok(())
}
```

## Error Handling

All tools follow a consistent error handling pattern:

- Successful operations return a JSON response with `"status": "success"`
- Failed operations return a JSON response with `"status": "error"` and an `"error"` field containing the error message

Example success response:

```json
{
  "filepath": "datadog_cache/monitors_1699999999_abcd1234.json",
  "summary": "Retrieved 42 monitors",
  "total_monitors": 42,
  "monitor_states": {
    "OK": 38,
    "Alert": 4
  },
  "alerting_count": 4,
  "status": "success"
}
```

Example error response:

```json
{
  "error": "Failed to get monitors: API error: 401 - Unauthorized",
  "status": "error"
}
```

## Configuration

### Environment Variables

- `DD_API_KEY` (required) - API authentication credential
- `DD_APP_KEY` (required) - Application-level authentication credential
- `DD_SITE` (optional) - Regional endpoint (defaults to "datadoghq.com")
  - US1: `datadoghq.com` (default)
  - US3: `us3.datadoghq.com`
  - US5: `us5.datadoghq.com`
  - EU: `datadoghq.eu`
  - AP1: `ap1.datadoghq.com`
  - US1-FED: `ddog-gov.com`

### Logging

Set the `RUST_LOG` environment variable to control log verbosity:

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Info, warnings, and errors (default)
- `debug` - Debug info and above
- `trace` - All logging

## Troubleshooting

### "DD_API_KEY not set" error

Make sure you have created a `.env` file with your credentials or set the environment variables.

### "API validation failed: 403" error

Check that your API and Application keys are correct and have the necessary permissions.

### Connection timeouts

The client has a 30-second timeout for HTTP requests. If you're experiencing timeouts, check your network connection and Datadog site configuration.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Comparison with Python Implementation

This Rust implementation provides:

- **Better Performance**: Async runtime with zero-cost abstractions
- **Type Safety**: Compile-time guarantees prevent many runtime errors
- **Smaller Memory Footprint**: More efficient resource usage
- **Single Binary**: No Python runtime required
- **Modular Design**: API client can be used independently

The tool interface and functionality remain compatible with the original Python implementation.
