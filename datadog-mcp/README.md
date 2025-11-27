# Datadog MCP Server

A high-performance [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server that connects AI assistants to Datadog's monitoring and observability platform. Built in Rust for speed, reliability, and token efficiency.

```
┌─────────────────┐     MCP/stdio      ┌──────────────────┐      HTTPS       ┌─────────────┐
│  Claude/LLM     │◄──────────────────►│  datadog-mcp     │◄────────────────►│  Datadog    │
│  Assistant      │   JSON or TOON     │  server          │   REST API       │  Platform   │
└─────────────────┘                    └──────────────────┘                  └─────────────┘
```

## Highlights

- **35+ MCP Tools** — Full coverage of Datadog's core APIs: metrics, monitors, dashboards, logs, synthetics, incidents, and more
- **TOON Output Format** — Optional [Token-Oriented Object Notation](https://github.com/toon-format/toon) reduces token usage by 30-60% compared to JSON
- **Async & Non-blocking** — Built on Tokio for high throughput and responsiveness
- **Automatic Retry** — Exponential backoff handles rate limits gracefully
- **Local Caching** — All API responses cached for analysis and replay
- **Modular Architecture** — Standalone `datadog-api` crate usable in other Rust projects

## Quick Start

### 1. Build

```bash
git clone <repository-url>
cd datadog-mcp
cargo build --release
```

### 2. Configure

```bash
export DD_API_KEY="your_api_key"
export DD_APP_KEY="your_app_key"
export DD_SITE="datadoghq.eu"  # or datadoghq.com, us3.datadoghq.com, etc.

# Optional: store credentials in your system keyring (macOS Keychain, Windows Credential Manager, Secret Service)
DD_PROFILE=default ./target/release/datadog-mcp --store-credentials
# Afterwards you can unset the env vars; the server will read from keyring first, then ~/.datadog-mcp/credentials.json, then env vars.
```

### 3. Run

```bash
./target/release/datadog-mcp --format toon
```

## Configuration

### Command-Line Options

| Option | Values | Default | Description |
|--------|--------|---------|-------------|
| `--format` | `json`, `toon` | `toon` | Output format for MCP responses |

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DD_API_KEY` | Yes | Datadog API key |
| `DD_APP_KEY` | Yes | Datadog Application key |
| `DD_SITE` | No | Datadog site (default: `datadoghq.com`) |
| `DD_PROFILE` | No | Credential profile name (used for keyring entry; default `default`) |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug`, `trace` |

### Credential Storage Order
1. System keyring (`datadog-mcp` service, profile `DD_PROFILE` or `default`)
2. `~/.datadog-mcp/credentials.json` (`{"api_key":"...","app_key":"...","site":"..."}`)
3. Environment variables (`DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`)

### Supported Datadog Sites

| Site | Domain |
|------|--------|
| US1 (default) | `datadoghq.com` |
| US3 | `us3.datadoghq.com` |
| US5 | `us5.datadoghq.com` |
| EU | `datadoghq.eu` |
| AP1 | `ap1.datadoghq.com` |
| US1-FED | `ddog-gov.com` |

## MCP Client Setup

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/path/to/datadog-mcp",
      "args": ["--format", "toon"],
      "env": {
        "DD_API_KEY": "your_api_key",
        "DD_APP_KEY": "your_app_key",
        "DD_SITE": "datadoghq.eu"
      }
    }
  }
}
```

### Claude Code

Add to your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "datadog": {
      "command": "/path/to/datadog-mcp",
      "args": ["--format", "toon"],
      "env": {
        "DD_API_KEY": "your_api_key",
        "DD_APP_KEY": "your_app_key",
        "DD_SITE": "datadoghq.eu"
      }
    }
  }
}
```

## Available Tools

### Metrics & Monitoring

| Tool | Description |
|------|-------------|
| `validate_api_key` | Verify API credentials |
| `get_metrics` | Query time series data |
| `search_metrics` | Find metrics by name pattern |
| `get_metric_metadata` | Retrieve metric metadata |
| `get_monitors` | List all monitors |
| `get_monitor` | Get specific monitor by ID |
| `create_monitor` | Create new monitor |
| `update_monitor` | Modify existing monitor |
| `delete_monitor` | Remove monitor |

### Dashboards

| Tool | Description |
|------|-------------|
| `get_dashboards` | List all dashboards |
| `get_dashboard` | Get dashboard by ID |
| `create_dashboard` | Create new dashboard |
| `update_dashboard` | Modify existing dashboard |
| `delete_dashboard` | Remove dashboard |

### Logs & Events

| Tool | Description |
|------|-------------|
| `search_logs` | Query log entries with filters |
| `get_events` | Retrieve system events |

### Infrastructure

| Tool | Description |
|------|-------------|
| `get_infrastructure` | Get host information |
| `get_tags` | Retrieve host tags |
| `get_kubernetes_deployments` | List K8s deployments |

### Synthetics Testing

| Tool | Description |
|------|-------------|
| `get_synthetics_tests` | List all synthetic tests |
| `get_synthetics_locations` | Get available test locations |
| `create_synthetics_test` | Create API/HTTP test |
| `update_synthetics_test` | Modify existing test |
| `trigger_synthetics_tests` | Run tests on-demand |

### Downtimes

| Tool | Description |
|------|-------------|
| `get_downtimes` | List scheduled downtimes |
| `create_downtime` | Schedule maintenance window |
| `cancel_downtime` | Cancel scheduled downtime |

### Security & Incidents

| Tool | Description |
|------|-------------|
| `get_security_rules` | Retrieve security monitoring rules |
| `get_incidents` | Access incident data |
| `get_slos` | Get Service Level Objectives |
| `get_notebooks` | Retrieve Datadog notebooks |

### Teams & Users

| Tool | Description |
|------|-------------|
| `get_teams` | List teams |
| `get_users` | List users |

### Utilities

| Tool | Description |
|------|-------------|
| `analyze_data` | Analyze cached data (summary, stats, trends) |
| `cleanup_cache` | Remove old cache files |

## Output Formats

### JSON (Traditional)

Standard JSON output, compatible with all systems:

```json
{
  "status": "success",
  "total_monitors": 42,
  "monitors": [
    {"id": 123, "name": "CPU Alert", "status": "OK"},
    {"id": 124, "name": "Memory Alert", "status": "Alert"}
  ]
}
```

### TOON (Token-Efficient)

[TOON format](https://github.com/toon-format/toon) optimizes for LLM consumption, reducing tokens by 30-60%:

```
status:success
total_monitors:42
monitors:[
  {id:123 name:"CPU Alert" status:OK}
  {id:124 name:"Memory Alert" status:Alert}
]
```

TOON is the default format. Use `--format json` if you need standard JSON output.

## Caching

All API responses are cached locally for offline analysis and debugging:

```
datadog_cache/
├── monitors_1700000000_a1b2c3d4.toon
├── dashboards_1700000001_e5f6g7h8.toon
└── logs_1700000002_i9j0k1l2.json
```

Cache files use the configured output format. Use `cleanup_cache` to remove old files:

```
cleanup_cache(older_than_hours: 24)
```

## Architecture

```
datadog-mcp/
├── src/
│   ├── main.rs          # Entry point, CLI parsing
│   ├── server.rs        # MCP server & tool handlers
│   ├── state.rs         # Server state & configuration
│   ├── output.rs        # JSON/TOON formatting
│   ├── cache.rs         # Response caching
│   ├── tools.rs         # Tool implementations (part 1)
│   ├── tools_part2.rs   # Tool implementations (part 2)
│   └── tool_inputs.rs   # Input type definitions
└── tests/
    └── integration_tests.rs
```

The server depends on the sibling `datadog-api` crate which provides:

- Type-safe Datadog API client
- Automatic retry with exponential backoff
- Support for all regional endpoints

## Development

```bash
# Build
cargo build

# Test
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- --format json

# Check formatting
cargo fmt --check

# Lint
cargo clippy
```

## Using datadog-api Independently

The API client can be used as a standalone library:

```rust
use datadog_api::{DatadogClient, DatadogConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;

    // Query metrics
    let metrics = client.query_metrics(
        "avg:system.cpu.user{*}",
        now - 3600,
        now
    ).await?;

    println!("{:?}", metrics);
    Ok(())
}
```

## Troubleshooting

| Error | Solution |
|-------|----------|
| `DD_API_KEY not set` | Set environment variables or create `.env` file |
| `403 Forbidden` | Check API/App key permissions in Datadog |
| `Connection timeout` | Verify `DD_SITE` matches your Datadog region |
| `Rate limited` | Automatic retry handles this; reduce request frequency if persistent |

## License

MIT

---

Built with Rust, [rmcp](https://github.com/modelcontextprotocol/rust-sdk), and [toon-rs](https://github.com/jimmystridh/toon-rs).
