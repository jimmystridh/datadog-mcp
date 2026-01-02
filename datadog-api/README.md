# datadog-api

A Rust client library for the Datadog API with type-safe access to monitors, dashboards, metrics, logs, synthetics, and more.

[![Crates.io](https://img.shields.io/crates/v/datadog-api.svg)](https://crates.io/crates/datadog-api)
[![Documentation](https://docs.rs/datadog-api/badge.svg)](https://docs.rs/datadog-api)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../LICENSE)

## Features

- **16 API modules** covering monitors, dashboards, metrics, logs, synthetics, events, infrastructure, downtimes, incidents, SLOs, security, notebooks, teams, users, and traces
- **Automatic retry** with exponential backoff for transient failures
- **Rate limiting** with client-side token bucket
- **Conditional requests** with ETag/If-Modified-Since support
- **Type-safe timestamps** with `TimestampSecs`, `TimestampMillis`, `TimestampNanos`
- **Secure credential storage** via system keyring (macOS Keychain, Windows Credential Manager, Secret Service)
- **All Datadog regions** supported (US1, US3, US5, EU, AP1, US1-FED)

## Installation

```toml
[dependencies]
datadog-api = "0.1"
```

To disable keyring support (for environments without a system keyring):

```toml
[dependencies]
datadog-api = { version = "0.1", default-features = false }
```

## Quick Start

```rust
use datadog_api::{DatadogClient, DatadogConfig};
use datadog_api::apis::MonitorsApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config from environment variables
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;

    // List all monitors
    let api = MonitorsApi::new(client);
    let monitors = api.list_monitors().await?;

    for monitor in monitors {
        println!("{}: {:?}",
            monitor.id.unwrap_or_default(),
            monitor.name.unwrap_or_default()
        );
    }

    Ok(())
}
```

## Configuration

### Environment Variables

```bash
export DD_API_KEY="your_api_key"
export DD_APP_KEY="your_app_key"
export DD_SITE="datadoghq.eu"  # Optional, defaults to datadoghq.com
```

### Programmatic Configuration

```rust
use datadog_api::{DatadogConfig, DatadogClient, HttpConfig, RetryConfig};

let config = DatadogConfig::new(
    "your_api_key".to_string(),
    "your_app_key".to_string(),
)
.with_site("datadoghq.eu".to_string())
.with_retry(RetryConfig {
    max_retries: 3,
    initial_backoff_ms: 100,
    max_backoff_ms: 10000,
    backoff_multiplier: 2.0,
})
.with_http(HttpConfig {
    timeout_secs: 30,
    pool_max_idle_per_host: 10,
    pool_idle_timeout_secs: 90,
    tcp_keepalive_secs: Some(60),
});

let client = DatadogClient::new(config)?;
```

### Credential Sources

The library loads credentials in this order:

1. **Keyring** (if `keyring` feature enabled): System credential storage
2. **File**: `~/.datadog-mcp/credentials.json`
3. **Environment**: `DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`

```rust
// Try all sources
let config = DatadogConfig::from_env_or_file()?;

// Or load from specific source
let config = DatadogConfig::from_env()?;  // Environment only
```

## API Modules

```rust
use datadog_api::apis::*;

let client = DatadogClient::new(config)?;

// Monitoring
let monitors = MonitorsApi::new(client.clone());
let dashboards = DashboardsApi::new(client.clone());
let slos = SLOsApi::new(client.clone());

// Metrics & Logs
let metrics = MetricsApi::new(client.clone());
let logs = LogsApi::new(client.clone());
let events = EventsApi::new(client.clone());

// Infrastructure
let infra = InfrastructureApi::new(client.clone());
let downtimes = DowntimesApi::new(client.clone());

// Testing
let synthetics = SyntheticsApi::new(client.clone());

// Security & Incidents
let security = SecurityApi::new(client.clone());
let incidents = IncidentsApi::new(client.clone());

// Organization
let teams = TeamsApi::new(client.clone());
let users = UsersApi::new(client.clone());
let notebooks = NotebooksApi::new(client.clone());

// APM
let traces = TracesApi::new(client.clone());
```

## Error Handling

```rust
use datadog_api::Error;

match api.get_monitor(12345).await {
    Ok(monitor) => println!("Found: {:?}", monitor.name),
    Err(e) => {
        if e.is_not_found() {
            println!("Monitor doesn't exist");
        } else if e.is_unauthorized() {
            println!("Check your API keys");
        } else if e.is_rate_limited() {
            println!("Too many requests, will retry");
        } else if e.is_retryable() {
            println!("Transient error, retry later");
        } else {
            println!("Error: {}", e);
        }
    }
}
```

## Timestamps

Type-safe timestamp handling to avoid unit confusion:

```rust
use datadog_api::{TimestampSecs, TimestampMillis};

// Current time
let now = TimestampSecs::now();

// Relative times
let yesterday = TimestampSecs::days_ago(1);
let last_hour = TimestampSecs::hours_ago(1);

// Convert between units
let millis: TimestampMillis = now.to_millis();

// Use in queries
let metrics = api.query_metrics(
    yesterday.as_secs(),
    now.as_secs(),
    "avg:system.cpu.user{*}"
).await?;
```

## Conditional Requests

Reduce bandwidth with ETag caching:

```rust
use datadog_api::CacheInfo;

// First request
let response = client.get_cached::<Monitor>("/api/v1/monitor/123", None).await?;
let cache_info = response.as_ref().map(|r| &r.cache_info);

// Subsequent request - returns None if unchanged (304)
let response = client.get_cached::<Monitor>("/api/v1/monitor/123", cache_info).await?;
match response {
    Some(r) => println!("Data changed: {:?}", r.data),
    None => println!("Not modified, use cached version"),
}
```

## Supported Datadog Sites

| Site | Domain | Usage |
|------|--------|-------|
| US1 (default) | `datadoghq.com` | Most users |
| US3 | `us3.datadoghq.com` | US region 3 |
| US5 | `us5.datadoghq.com` | US region 5 |
| EU | `datadoghq.eu` | European Union |
| AP1 | `ap1.datadoghq.com` | Asia Pacific |
| US1-FED | `ddog-gov.com` | US Government |

## License

MIT - See [LICENSE](../LICENSE) for details.

## Related

- [datadog-mcp](../) - MCP server that uses this library to connect AI assistants to Datadog
