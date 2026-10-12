# Datadog MCP Server

A high-performance [Model Context Protocol](https://modelcontextprotocol.io/) (MCP) server that connects AI assistants to Datadog's monitoring and observability platform. Built in Rust for speed, reliability, and token efficiency.

```
┌─────────────────┐     MCP/stdio      ┌──────────────────┐      HTTPS       ┌─────────────┐
│  Claude/LLM     │◄──────────────────►│  datadog-mcp     │◄────────────────►│  Datadog    │
│  Assistant      │   JSON or TOON     │  server          │   REST API       │  Platform   │
└─────────────────┘                    └──────────────────┘                  └─────────────┘
```

## Highlights

- **49 MCP Tools** — Broad coverage of Datadog's core APIs: metrics, monitors, database monitoring, Slack, dashboards, logs, synthetics, incidents, and more
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
git clone https://github.com/jimmystridh/datadog-mcp.git
cd datadog-mcp
cargo build --release
```

On macOS, install the binary with a persistent code-signing identity so Keychain access remains
trusted after upgrades:

```bash
security find-identity -v -p codesigning
./scripts/install-macos.sh "Apple Development: Your Name (TEAMID)"
```

The first signed installation can require one Keychain approval. Choose **Always Allow**. Future
installations signed with the same identity and identifier are recognized as the same application.
If no signing identity is listed, create a Code Signing certificate in Keychain Access first.
The identity can alternatively be supplied through `DATADOG_MCP_CODESIGN_IDENTITY`.

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
| `SWEET_COOKIE_CHROME_PROFILE` | No | Chrome profile directory used for Datadog UI-only tools, such as `Profile 1` |
| `SWEET_COOKIE_EDGE_PROFILE` | No | Edge profile directory used for Datadog UI-only tools |
| `SWEET_COOKIE_FIREFOX_PROFILE` | No | Firefox profile used for Datadog UI-only tools |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug`, `trace` |

MCP initialization does not access credentials or call Datadog. Credentials and the API client are
initialized once, on the first Datadog tool call, so a Keychain prompt or slow network cannot cause
the MCP startup handshake to time out.

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
| `search_monitors` | Search monitors with pagination and sorting |
| `get_monitor` | Get monitor configuration, group states, status timestamps, and active matching downtimes |
| `create_monitor` | Create new monitor |
| `update_monitor` | Modify existing monitor |
| `resolve_monitor_group` | Resolve one exact triggered group using Datadog's legacy endpoint |
| `delete_monitor` | Remove monitor |

`get_monitor` requests all group states and active matching downtimes by default. Use
`group_states` to limit the returned states (`all`, `alert`, `warn`, or `no data`), pass an empty
array to omit group details, or set `with_downtimes` to `false` to omit downtime details.

`resolve_monitor_group` requires `--allow-write`, an exact group key copied from
`get_monitor.data.state.groups`, and both monitor read and write permissions. It rejects
Datadog's special `ALL_GROUPS` value, verifies the group before and after the write, and does not
send a resolution when the group is already `OK`. The underlying Datadog endpoint is undocumented
and unversioned, so the result identifies it as `legacy_undocumented` and callers should treat the
tool as experimental. Resolving a group may emit a recovery notification.

### Slack Notifications

| Tool | Description |
|------|-------------|
| `get_slack_channels` | List channels configured for a Slack account/workspace |
| `get_slack_channel` | Get one channel and its alert display settings |
| `create_slack_channel` | Add a channel for monitor and event notifications |
| `update_slack_channel` | Rename a configured channel or update its display settings |
| `delete_slack_channel` | Remove a configured channel without changing monitor messages |
| `audit_slack_monitor_notifications` | Correlate monitor `@slack-…` handles with configured channels |

The Slack workspace must already be installed through Datadog's OAuth-based integration tile;
the public API only manages channel mappings within that installation. Supply the Datadog Slack
account name shown on the integration tile. Channel create, update, and delete operations require
`--allow-write` and an application key whose owner has **Integrations Manage**. Reads and the audit
remain available in the default read-only mode.

The audit recognizes both `@slack-<channel>` and
`@slack-<account>-<channel>`, reports configured-but-unused channels, preserves dynamic handles such
as `@slack-{{owner.name}}` separately, and labels other handles as unmatched for the selected
account rather than assuming they are invalid globally. Removing a channel does not rewrite any
monitor message, so run the audit before deletion.

### Dashboards

| Tool | Description |
|------|-------------|
| `get_dashboards` | List all dashboards |
| `get_dashboard` | Get dashboard by ID |
| `create_dashboard` | Create new dashboard |
| `update_dashboard` | Modify existing dashboard |
| `delete_dashboard` | Remove dashboard |

### Database Monitoring

| Tool | Description |
|------|-------------|
| `get_database_query_metrics` | Rank query signatures by duration, executions, or other DBM metrics |
| `search_database_query_samples` | Read active-session snapshots or explain-plan samples with full event attributes |

These tools use Datadog's [documented DBM API](https://docs.datadoghq.com/database_monitoring/guide/build_apps_with_dbm_api/)
with API-key authentication. No browser session is needed. Sample searches require an **unscoped
application key** because `built_in_features` cannot be assigned to a scoped key, and the caller
needs Database Monitoring access. Scalar metrics need metric-query permissions. DBM must be
enabled on the database integration.

For SQL Server, call `get_database_query_metrics` with
`sum:sqlserver.queries.time{env:prod} by {query_signature,database_instance}`, Unix-second
`from_timestamp` and `to_timestamp`, and `aggregator: "sum"` to rank total execution duration.
Use `sqlserver.queries.count` for execution counts. Returned group and numeric columns share row
indices. A signature's SQL statement can be retrieved with `search_database_query_samples`,
filtering `env:prod @db.query_signature:<signature>` and `sample_type: "plan"`.

For the Active Connections view, search `env:prod @sqlserver.session_id:85` with
`sample_type: "activity"` (the default). Full details are at
`data.result.events[].event.custom`: SQL text, user, application, wait events, session IDs,
and engine-specific blocking/transaction fields when collected. Results are historical snapshots;
they do not execute SQL against the database. Each search returns at most `limit` newest records
(default 100, maximum 1000). Narrow the filter or time range for additional records. Raw paging
metadata is preserved, but resuming a page is not currently supported. SQL statements and plans
are never written to the optional response cache.

### Logs & Events

| Tool | Description |
|------|-------------|
| `search_logs` | Query one cursor-paginated page of log entries |
| `get_events` | Retrieve system events |
| `create_event` | Create a Datadog event |
| `get_event` | Get a specific event by ID |

### Infrastructure

| Tool | Description |
|------|-------------|
| `get_infrastructure` | Get host information |
| `get_tags` | Retrieve host tags |
| `get_kubernetes_deployments` | List K8s deployments |
| `get_workload_scaling_recommendations` | Get Datadog's Kubernetes workload-scaling report using a local browser session |

Datadog does not expose the workload-scaling report through its public API. The
`get_workload_scaling_recommendations` tool therefore uses `cookie-scoop` to read an existing local
Datadog browser session and calls the read-only, undocumented
`/api/ui/orchestration/autoscaling/workloads/eligible` endpoint. Chrome is the default browser.
Use `browser_profile` or the corresponding `SWEET_COOKIE_*_PROFILE` environment variable when the
Datadog login is not in the browser's default profile. Chrome profile directory names such as
`Default` or `Profile 1` are shown on `chrome://version`; filesystem paths are not accepted through
the MCP tool.

Cookies are held in memory for up to 15 minutes to avoid repeated OS credential prompts, never
returned in MCP responses, logged, or written to the response cache. A cached session is refreshed
automatically after an unauthorized response; set `refresh_browser_session` to refresh it sooner.
The endpoint is not covered by Datadog's public compatibility guarantees and may change without
notice.

### Synthetics Testing

| Tool | Description |
|------|-------------|
| `get_synthetics_tests` | List all synthetic tests |
| `get_synthetics_locations` | Get available test locations |
| `create_synthetics_test` | Create API/HTTP test |
| `update_synthetics_test` | Modify existing test |
| `trigger_synthetics_tests` | Run tests on-demand |
| `delete_synthetics_tests` | Delete one or more tests |

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
  "summary": "Retrieved 42 monitors",
  "filepath": null,
  "total_monitors": 42,
  "monitor_states": {"OK": 41, "Alert": 1},
  "alerting_count": 1,
  "data": [
    {"id": 123, "name": "CPU Alert", "overall_state": "OK"},
    {"id": 124, "name": "Memory Alert", "overall_state": "Alert"}
  ]
}
```

Every successful response uses the same envelope: `status`, a human-readable `summary`, a `filepath` containing the cached path or `null`, optional tool-specific metadata, and the Datadog payload in `data` when returned. Errors use `status: "error"` with an `error` message.

### TOON (Token-Efficient)

[TOON format](https://github.com/toon-format/toon) optimizes for LLM consumption, reducing tokens by 30-60%:

```
alerting_count: 1
data[2]{id,name,overall_state}:
  123,CPU Alert,OK
  124,Memory Alert,Alert
filepath: null
monitor_states:
  Alert: 1
  OK: 41
status: success
summary: Retrieved 42 monitors
total_monitors: 42
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
│   ├── src/
│   │   ├── main.rs        # Entry point and CLI parsing
│   │   ├── server.rs      # MCP registration, annotations, and write policy
│   │   ├── state.rs       # Shared runtime context
│   │   ├── response.rs    # Typed response envelope and rendering
│   │   ├── output.rs      # JSON and TOON formatting
│   │   ├── cache.rs       # Sandboxed response cache
│   │   ├── tool_inputs.rs # MCP input schemas
│   │   └── tools/         # Tool implementations by domain
│   └── tests/
│       └── tool_http/    # Domain-split HTTP integration tests
│
└── datadog-api/          # Standalone Rust API client
    └── src/
        ├── client.rs             # HTTP client and end-to-end retry deadlines
        ├── rate_limit.rs         # Shared rate limiting
        ├── pagination.rs         # Validated pagination types
        ├── resource_documents.rs # Lossless update documents
        ├── apis/         # API modules (monitors, dashboards, etc.)
        └── models/       # Request/response types
```

The MCP server depends on the [`datadog-api`](datadog-api/) crate, which can also be used independently in other Rust projects. See the [datadog-api README](datadog-api/README.md) for library usage.

## Development

```bash
# Match the CI quality gates
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo check --locked --workspace --all-targets --no-default-features
cargo +1.88.0 check --locked --workspace --all-targets --all-features
cargo test --locked --all --tests -- --nocapture
cargo audit --deny warnings
cargo deny check

# Run locally with JSON responses and debug logging
RUST_LOG=debug cargo run --locked -- --format json
```

## Using datadog-api Independently

The API client can be used as a standalone library:

```rust
use datadog_api::apis::MetricsApi;
use datadog_api::{DatadogClient, DatadogConfig, TimestampSecs};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;
    let api = MetricsApi::new(client);
    let now = TimestampSecs::now();

    let metrics = api
        .query_metrics(
            TimestampSecs::hours_ago(1).as_secs(),
            now.as_secs(),
            "avg:system.cpu.user{*}",
        )
        .await?;

    println!("{metrics:?}");
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
| `no Datadog browser cookies were found` | Sign in to the configured Datadog site and select the correct browser/profile |
| Browser session unauthorized | Sign in again, then call the tool with `refresh_browser_session: true` |

## License

MIT

---

Built with Rust, [rmcp](https://github.com/modelcontextprotocol/rust-sdk), and [toon-rs](https://github.com/jimmystridh/toon-rs).
