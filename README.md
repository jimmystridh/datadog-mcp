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
- **Safe Retry** — Read-only requests retry transient failures and honor Datadog rate-limit reset headers; writes are never retried automatically
- **Optional Local Caching** — Non-sensitive responses can be cached explicitly for analysis and replay
- **Read-Only by Default** — Datadog mutations require the explicit `--allow-write` startup flag
- **Modular Architecture** — Standalone `datadog-api` crate usable in other Rust projects

## Quick Start

### 1. Build

Requires Rust 1.88 or newer.

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
# Afterwards you can unset the env vars; the server will read from keyring, then ~/.datadog-mcp/credentials.json.
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
| `--allow-write` | flag | off | Enable tools that mutate Datadog resources |
| `--cache-responses` | flag | off | Cache non-sensitive API responses locally |
| `--cache-retention-hours` | integer | `24` | Remove older cache files at startup |
| `--cache-max-mib` | integer | `100` | Bound the cache; remove oldest files first |
| `--store-credentials` | flag | off | Store env/file credentials in the system keyring and exit |

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DD_API_KEY` | Yes | Datadog API key |
| `DD_APP_KEY` | Yes | Datadog Application key |
| `DD_SITE` | No | Datadog site (default: `datadoghq.com`) |
| `DD_PROFILE` | No | Credential profile name (used for keyring entry; default `default`) |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug`, `trace` |

### Credential Storage Order
1. Environment variables (`DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`)
2. System keyring (`datadog-mcp` service, profile `DD_PROFILE` or `default`)
3. `~/.datadog-mcp/credentials.json` (`{"api_key":"...","app_key":"...","site":"..."}`; mode `0600` required on Unix)

If either key environment variable is set, environment loading is authoritative and a partial pair is rejected rather than silently falling back.

### Supported Datadog Sites

| Site | Domain |
|------|--------|
| US1 (default) | `datadoghq.com` |
| US3 | `us3.datadoghq.com` |
| US5 | `us5.datadoghq.com` |
| EU | `datadoghq.eu` |
| AP1 | `ap1.datadoghq.com` |
| AP2 | `ap2.datadoghq.com` |
| UK1 | `uk1.datadoghq.com` |
| US1-FED | `ddog-gov.com` |
| US2-FED | `us2.ddog-gov.com` |

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
| `search_metrics` | Find active metrics by name and optional active-since timestamp |
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
| `search_logs` | Query one cursor-paginated page of log entries |
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
| `get_incidents` | Access one offset-paginated incident page |
| `get_slos` | Get Service Level Objectives |
| `get_notebooks` | Retrieve Datadog notebooks |

### Teams & Users

| Tool | Description |
|------|-------------|
| `get_teams` | List one page of teams |
| `get_users` | List one page of users |

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

Caching is disabled by default. Enable it with `--cache-responses`; results remain available inline through MCP `structuredContent` whether caching is enabled or not.

```
~/.cache/datadog-mcp/
├── monitors_1700000000_a1b2c3d4.toon
└── dashboards_1700000001_e5f6g7h8.toon
```

The cache directory is mode `0700` and files are mode `0600` on Unix. Logs, users, events, incidents, and security-rule responses are never cached. Cache analysis is confined to this directory and refuses files larger than 10 MiB. The startup retention/size limits run automatically; `cleanup_cache` is also available when write mode is enabled.

```
cleanup_cache(older_than_hours: 24)
```

## Architecture

```
.
├── datadog-mcp/          # MCP server (this binary)
│   └── src/
│       ├── main.rs       # Entry point, CLI parsing
│       ├── server.rs     # MCP server & tool handlers
│       ├── state.rs      # Server state & configuration
│       ├── cache.rs      # Response caching
│       └── tools/        # Tool implementations by domain
│
└── datadog-api/          # Standalone Rust API client
    └── src/
        ├── client.rs     # HTTP client with retry & rate limiting
        ├── apis/         # API modules (monitors, dashboards, etc.)
        └── models/       # Request/response types
```

The MCP server depends on the [`datadog-api`](datadog-api/) crate, which can also be used independently in other Rust projects. See the [datadog-api README](datadog-api/README.md) for library usage.

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
