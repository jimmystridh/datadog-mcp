# Datadog MCP Improvements Plan

This document outlines planned improvements for the Datadog MCP Rust implementation based on a comprehensive code quality review. Overall score: **7.9/10** - Production-ready with room for improvement.

## Priority Levels

- **P0**: Critical - Do first
- **P1**: High - This week
- **P2**: Medium - Next sprint
- **P3**: Low - Backlog

---

## 1. Code Organization (Score: 8/10 → Target: 9.5/10)

### 1.1 Reorganize Tools by Domain [P1]

**Current State**: `tools.rs` (543 lines) and `tools_part2.rs` (1,121 lines) are split arbitrarily.

**Target Structure**:
```
src/tools/
├── mod.rs           # Re-exports all tool modules
├── metrics.rs       # get_metrics, search_metrics, get_metric_metadata
├── monitors.rs      # get_monitors, get_monitor, create_monitor, update_monitor, delete_monitor
├── dashboards.rs    # get_dashboards, get_dashboard, create_dashboard, update_dashboard, delete_dashboard
├── logs.rs          # search_logs
├── events.rs        # get_events
├── infrastructure.rs # get_infrastructure, get_tags, get_kubernetes_deployments
├── downtimes.rs     # get_downtimes, create_downtime, cancel_downtime
├── synthetics.rs    # get_synthetics_tests, get_synthetics_locations, create_synthetics_test, update_synthetics_test, trigger_synthetics_tests
├── security.rs      # get_security_rules
├── incidents.rs     # get_incidents
├── slos.rs          # get_slos
├── notebooks.rs     # get_notebooks
├── teams.rs         # get_teams
├── users.rs         # get_users
├── cache_tools.rs   # analyze_data, cleanup_cache
└── validation.rs    # validate_api_key
```

**Effort**: 4 hours

### 1.2 Split Server Registration [P2]

**Current State**: `server.rs` (617 lines) contains both initialization and all tool registrations.

**Target Structure**:
```
src/
├── server.rs        # MCP server initialization, handlers
└── tools_registry.rs # Tool method implementations that call into tools/
```

**Effort**: 2 hours

### 1.3 Consolidate Tool Inputs [P3]

**Current State**: `tool_inputs.rs` duplicates structure from API models.

**Action**: Evaluate whether tool inputs can derive from or share types with API models.

**Effort**: 3 hours

---

## 2. Code Duplication (Score: 6.5/10 → Target: 9/10)

### 2.1 Extract Tool Response Macro [P0]

**Current State**: ~40 identical error handling patterns across tools:
```rust
match result {
    Ok(data) => {
        let filepath = store_data(&data, "prefix", ctx.output_format).await?;
        info!("Retrieved ...");
        Ok(json!({
            "filepath": filepath,
            "summary": format!("..."),
            "status": "success",
        }))
    }
    Err(e) => {
        error!("Failed: {}", e);
        Ok(json!({
            "error": format!("Failed: {}", e),
            "status": "error",
        }))
    }
}
```

**Solution**: Create `src/tools/response.rs`:
```rust
/// Helper for consistent tool responses with caching
pub async fn tool_success<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
    summary: impl Into<String>,
) -> anyhow::Result<Value> {
    let filepath = store_data(data, prefix, format).await?;
    let summary = summary.into();
    info!("{}", summary);
    Ok(json!({
        "filepath": filepath,
        "summary": summary,
        "status": "success",
    }))
}

pub fn tool_error(operation: &str, error: impl std::fmt::Display) -> Value {
    let msg = format!("{}: {}", operation, error);
    error!("{}", msg);
    json!({
        "error": msg,
        "status": "error",
    })
}

/// Macro for the common match pattern
#[macro_export]
macro_rules! tool_response {
    ($result:expr, $prefix:expr, $ctx:expr, $summary:expr) => {
        match $result {
            Ok(data) => $crate::tools::response::tool_success(
                &data, $prefix, $ctx.output_format, $summary
            ).await,
            Err(e) => Ok($crate::tools::response::tool_error($prefix, e)),
        }
    };
}
```

**Impact**: Reduces ~500 lines of duplicated code

**Effort**: 2 hours

### 2.2 Add ToolContext API Helpers [P1]

**Current State**: API client construction repeated 35+ times:
```rust
let api = MetricsApi::new((*ctx.client).clone());
```

**Solution**: Add helper methods to `ToolContext`:
```rust
impl ToolContext {
    pub fn metrics_api(&self) -> MetricsApi {
        MetricsApi::new((*self.client).clone())
    }

    pub fn monitors_api(&self) -> MonitorsApi {
        MonitorsApi::new((*self.client).clone())
    }

    pub fn dashboards_api(&self) -> DashboardsApi {
        DashboardsApi::new((*self.client).clone())
    }

    // ... all 16 API types
}
```

**Effort**: 1 hour

---

## 3. Error Handling (Score: 8/10 → Target: 9.5/10)

### 3.1 Add Specific MCP Error Codes [P2]

**Current State**: All errors use generic `-32603`.

**Solution**: Create error code mapping in `src/errors.rs`:
```rust
pub enum McpErrorCode {
    ParseError = -32700,      // Invalid JSON-RPC
    InvalidRequest = -32600,  // Invalid request
    MethodNotFound = -32601,  // Unknown method
    InvalidParams = -32602,   // Parameter validation failed
    InternalError = -32603,   // Internal server error
    ConfigError = -32000,     // Configuration problem
    ApiError = -32001,        // Datadog API returned error
    NetworkError = -32002,    // Network/connectivity issue
    RateLimited = -32003,     // Rate limit exceeded
    NotFound = -32004,        // Resource not found
}

impl McpErrorCode {
    pub fn to_error_data(self, message: String) -> ErrorData {
        ErrorData::new(ErrorCode(self as i32), message, None)
    }
}
```

**Effort**: 2 hours

### 3.2 Reduce Unwrap Usage [P2]

**Current State**: 78 uses of unwrap/expect across codebase.

**Action Items**:
- [ ] Audit all unwraps in non-test code
- [ ] Replace with proper error handling or `unwrap_or_default()` where safe
- [ ] Document remaining unwraps that are provably safe

**Target**: < 20 unwraps in non-test code

**Effort**: 3 hours

### 3.3 Preserve Error Context [P3]

**Current State**: Some errors lose original context:
```rust
.unwrap_or_else(|_| "Failed to read error body"...)
```

**Solution**: Use `anyhow` context or log original error before simplifying.

**Effort**: 1 hour

---

## 4. Type Safety (Score: 8/10 → Target: 9.5/10)

### 4.1 Implement Type-Safe IDs [P1]

**Current State**: IDs are raw strings/integers:
```rust
pub async fn get_dashboard(&self, dashboard_id: String) -> Result<Dashboard>
pub async fn get_monitor(&self, monitor_id: i64) -> Result<Monitor>
```

**Solution**: Create newtype wrappers in `datadog-api/src/ids.rs`:
```rust
use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $inner);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<$inner> for $name {
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }
    };
}

define_id!(DashboardId, String);
define_id!(MonitorId, i64);
define_id!(DowntimeId, i64);
define_id!(SyntheticsTestId, String);
define_id!(IncidentId, String);
define_id!(SloId, String);
define_id!(NotebookId, i64);
define_id!(TeamId, String);
define_id!(UserId, String);
```

**Effort**: 3 hours

### 4.2 Replace Loose Value Usage [P2]

**Current State**: `tool_inputs.rs` uses `serde_json::Value` for options.

**Solution**: Create typed Options structs:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitorOptions {
    pub thresholds: Option<MonitorThresholds>,
    pub notify_no_data: Option<bool>,
    pub evaluation_delay: Option<i64>,
    // ... other typed fields
}
```

**Effort**: 4 hours

### 4.3 Create Timestamp Newtype [P3]

**Current State**: Raw `i64` for timestamps with no unit indication.

**Solution**:
```rust
/// Unix timestamp in seconds
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimestampSecs(pub i64);

impl TimestampSecs {
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp())
    }

    pub fn from_millis(ms: i64) -> Self {
        Self(ms / 1000)
    }
}
```

**Effort**: 2 hours

---

## 5. Test Coverage (Score: 7/10 → Target: 9/10)

### 5.1 Add HTTP Mock Testing [P1]

**Current State**: No HTTP-level testing; relies on live API.

**Solution**: Add `wiremock` for HTTP mocking:

```toml
# Cargo.toml [dev-dependencies]
wiremock = "0.6"
```

```rust
// tests/mock_api_tests.rs
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_get_monitors_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .and(header("DD-API-KEY", "test-key"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!([{"id": 1, "name": "Test Monitor"}])))
        .mount(&mock_server)
        .await;

    let config = DatadogConfig {
        api_key: "test-key".into(),
        app_key: "test-app".into(),
        site: mock_server.uri().replace("http://", ""),
    };

    let client = DatadogClient::new(config).unwrap();
    let api = MonitorsApi::new(client);
    let result = api.list_monitors().await.unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, Some("Test Monitor".into()));
}
```

**Test Coverage Targets**:
- [ ] All API endpoints (16 modules × ~3 methods = ~48 tests)
- [ ] Error responses (401, 403, 404, 429, 500)
- [ ] Retry behavior
- [ ] Timeout handling

**Effort**: 8 hours

### 5.2 Add Tool Integration Tests [P2]

**Current State**: No tests for MCP tool execution.

**Solution**: Create `tests/tool_tests.rs`:
```rust
#[tokio::test]
async fn test_validate_api_key_tool() {
    let mock_server = setup_mock_server().await;
    let ctx = create_test_context(&mock_server).await;

    let result = tools::validate_api_key(ctx).await.unwrap();

    assert_eq!(result["status"], "success");
    assert_eq!(result["valid"], true);
}
```

**Effort**: 6 hours

### 5.3 Fix Test Isolation [P0]

**Current State**: Environment variable tests pollute each other.

**Solution**: Use `serial_test` crate or test-specific env handling:
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_from_env_default_site() {
    // Save and restore env vars
    let saved = std::env::var("DD_SITE").ok();
    std::env::remove_var("DD_SITE");

    // ... test ...

    if let Some(v) = saved {
        std::env::set_var("DD_SITE", v);
    }
}
```

**Effort**: 1 hour

### 5.4 Add Property-Based Tests [P3]

**Solution**: Use `proptest` for serialization round-trip testing:
```rust
proptest! {
    #[test]
    fn monitor_roundtrip(name in ".*", id in 0i64..1000000) {
        let monitor = Monitor { id: Some(id), name: Some(name.clone()), ..Default::default() };
        let json = serde_json::to_string(&monitor).unwrap();
        let parsed: Monitor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, Some(name));
    }
}
```

**Effort**: 3 hours

---

## 6. Performance (Score: 7.5/10 → Target: 9/10)

### 6.1 Add HTTP Compression [P1]

**Current State**: No compression headers.

**Solution**: Update `client.rs`:
```rust
let client = reqwest::Client::builder()
    .default_headers(headers)
    .gzip(true)           // Accept gzip responses
    .deflate(true)        // Accept deflate responses
    .pool_max_idle_per_host(10)
    .timeout(Duration::from_secs(30))
    .build()?;
```

**Impact**: 60-80% reduction in response size for large payloads.

**Effort**: 30 minutes

### 6.2 Optimize Tokio Features [P1]

**Current State**: `tokio = { features = ["full"] }` includes unnecessary features.

**Solution**:
```toml
# Before
tokio = { version = "1.40", features = ["full"] }

# After
tokio = { version = "1.40", features = ["macros", "rt-multi-thread", "io-util", "sync", "time", "fs"] }
```

**Impact**: ~1-2MB binary size reduction.

**Effort**: 30 minutes

### 6.3 Add Conditional Requests [P2]

**Solution**: Support ETag/Last-Modified for cacheable endpoints:
```rust
pub struct CachedResponse<T> {
    pub data: T,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl DatadogClient {
    pub async fn get_with_cache<T>(&self, endpoint: &str, etag: Option<&str>) -> Result<Option<CachedResponse<T>>> {
        let mut req = self.client.get(url);
        if let Some(etag) = etag {
            req = req.header("If-None-Match", etag);
        }

        let resp = req.send().await?;
        if resp.status() == 304 {
            return Ok(None); // Not modified
        }
        // ... parse and return with headers
    }
}
```

**Effort**: 4 hours

### 6.4 Add Pagination Helpers [P2]

**Solution**: Generic pagination support:
```rust
pub struct PageRequest {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: Option<i64>,
    pub next_page: Option<i32>,
}

impl<T> PaginatedResponse<T> {
    pub fn has_more(&self) -> bool {
        self.next_page.is_some()
    }
}
```

**Effort**: 3 hours

### 6.5 Make Connection Pool Configurable [P3]

**Solution**: Add to config:
```rust
pub struct DatadogConfig {
    // ... existing fields
    pub pool_max_idle: Option<usize>,
    pub timeout_secs: Option<u64>,
}
```

**Effort**: 1 hour

---

## 7. Security (Score: 7.5/10 → Target: 9.5/10)

### 7.1 Set Cache File Permissions [P0]

**Current State**: Cache files use default umask (potentially world-readable).

**Solution**: Update `cache.rs`:
```rust
#[cfg(unix)]
async fn set_secure_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o600);
    tokio::fs::set_permissions(path, perms).await?;
    Ok(())
}

pub async fn store_data<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
) -> Result<String> {
    // ... write file ...

    #[cfg(unix)]
    set_secure_permissions(Path::new(&filepath)).await?;

    Ok(filepath)
}
```

**Effort**: 1 hour

### 7.2 Add Secret Zeroing [P1]

**Solution**: Use `zeroize` crate:
```toml
# Cargo.toml
zeroize = { version = "1.8", features = ["derive"] }
```

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

pub struct DatadogConfig {
    pub api_key: SecretString,
    pub app_key: SecretString,
    pub site: String,
}
```

**Effort**: 2 hours

### 7.3 Sanitize Error Responses [P2]

**Solution**: Create error sanitizer:
```rust
pub fn sanitize_api_error(error: &Error) -> String {
    match error {
        Error::ApiError { status, message } => {
            // Don't expose full API response which might contain sensitive data
            format!("Datadog API error (status {})", status)
        }
        Error::HttpError(e) => {
            // Don't expose URLs which might contain query params
            "HTTP request failed".to_string()
        }
        _ => error.to_string()
    }
}
```

**Effort**: 2 hours

### 7.4 Add Request Logging Sanitization [P2]

**Solution**: Ensure secrets aren't logged:
```rust
impl DatadogClient {
    fn log_request(&self, method: &str, url: &str) {
        // Redact any API keys that might be in query strings
        let safe_url = url
            .replace(&self.config.api_key.expose(), "[REDACTED]")
            .replace(&self.config.app_key.expose(), "[REDACTED]");
        tracing::debug!(method, url = %safe_url, "API request");
    }
}
```

**Effort**: 1 hour

### 7.5 Add Client-Side Rate Limiting [P3]

**Solution**: Simple token bucket:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RateLimiter {
    requests_per_second: u32,
    last_request: AtomicU64,
}

impl RateLimiter {
    pub async fn wait(&self) {
        let min_interval = Duration::from_secs(1) / self.requests_per_second;
        // ... implement token bucket
    }
}
```

**Effort**: 3 hours

---

## 8. Documentation (Score: 8.5/10 → Target: 9.5/10)

### 8.1 Add Examples Directory [P2]

**Structure**:
```
examples/
├── basic_usage.rs       # Simple API calls
├── error_handling.rs    # Handling different error types
├── custom_config.rs     # Configuration options
├── standalone_api.rs    # Using datadog-api without MCP
└── batch_operations.rs  # Bulk operations pattern
```

**Effort**: 4 hours

### 8.2 Add Architecture Documentation [P2]

**Create**: `docs/ARCHITECTURE.md`
- Request flow diagram
- Component interactions
- Cache behavior
- Error handling strategy

**Effort**: 3 hours

### 8.3 Document Internal Modules [P3]

Add module-level documentation to:
- [ ] `cache.rs`
- [ ] `state.rs`
- [ ] `output.rs`
- [ ] All files in `tools/` (after reorganization)

**Effort**: 2 hours

### 8.4 Add Security Documentation [P3]

**Create**: `docs/SECURITY.md`
- Credential handling
- What's logged/cached
- Network security
- Threat model

**Effort**: 2 hours

---

## 9. Dependencies (Score: 8/10 → Target: 9/10)

### 9.1 Add Feature Flags [P2]

**Solution**:
```toml
[features]
default = ["retries", "toon"]
retries = ["reqwest-retry", "reqwest-middleware"]
toon = ["dep:toon"]
full = ["retries", "toon"]
```

**Effort**: 2 hours

### 9.2 Add Dev Dependencies [P1]

```toml
[dev-dependencies]
wiremock = "0.6"
serial_test = "3.0"
proptest = "1.4"
criterion = "0.5"  # For benchmarks
```

**Effort**: 1 hour

### 9.3 Add Benchmarks [P3]

**Create**: `benches/`
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_toon_encoding(c: &mut Criterion) {
    let data = create_large_response();
    c.bench_function("toon_encode", |b| {
        b.iter(|| toon::encode_to_string(&data, &Options::default()))
    });
}
```

**Effort**: 3 hours

---

## Implementation Schedule

### Week 1 (P0 + Quick Wins)

| Task | Effort | File(s) |
|------|--------|---------|
| 2.1 Extract tool response macro | 2h | `src/tools/response.rs` |
| 5.3 Fix test isolation | 1h | `tests/` |
| 7.1 Set cache file permissions | 1h | `src/cache.rs` |
| 6.1 Add HTTP compression | 30m | `datadog-api/src/client.rs` |
| 6.2 Optimize tokio features | 30m | `Cargo.toml` |
| **Total** | **5h** | |

### Week 2 (P1)

| Task | Effort | File(s) |
|------|--------|---------|
| 1.1 Reorganize tools by domain | 4h | `src/tools/` |
| 2.2 Add ToolContext API helpers | 1h | `src/state.rs` |
| 4.1 Implement type-safe IDs | 3h | `datadog-api/src/ids.rs` |
| 5.1 Add HTTP mock testing | 8h | `tests/mock_api_tests.rs` |
| 7.2 Add secret zeroing | 2h | `datadog-api/src/config.rs` |
| 9.2 Add dev dependencies | 1h | `Cargo.toml` |
| **Total** | **19h** | |

### Week 3 (P2)

| Task | Effort | File(s) |
|------|--------|---------|
| 1.2 Split server registration | 2h | `src/server.rs` |
| 3.1 Add specific MCP error codes | 2h | `src/errors.rs` |
| 3.2 Reduce unwrap usage | 3h | Various |
| 4.2 Replace loose Value usage | 4h | `src/tool_inputs.rs` |
| 5.2 Add tool integration tests | 6h | `tests/tool_tests.rs` |
| 6.3 Add conditional requests | 4h | `datadog-api/src/client.rs` |
| 6.4 Add pagination helpers | 3h | `datadog-api/src/pagination.rs` |
| 7.3 Sanitize error responses | 2h | `src/errors.rs` |
| 7.4 Add request logging sanitization | 1h | `datadog-api/src/client.rs` |
| 8.1 Add examples directory | 4h | `examples/` |
| 8.2 Add architecture documentation | 3h | `docs/ARCHITECTURE.md` |
| 9.1 Add feature flags | 2h | `Cargo.toml` |
| **Total** | **36h** | |

### Week 4+ (P3 - Backlog)

| Task | Effort |
|------|--------|
| 1.3 Consolidate tool inputs | 3h |
| 3.3 Preserve error context | 1h |
| 4.3 Create timestamp newtype | 2h |
| 5.4 Add property-based tests | 3h |
| 6.5 Make connection pool configurable | 1h |
| 7.5 Add client-side rate limiting | 3h |
| 8.3 Document internal modules | 2h |
| 8.4 Add security documentation | 2h |
| 9.3 Add benchmarks | 3h |
| **Total** | **20h** |

---

## Success Metrics

After completing all improvements:

| Category | Current | Target |
|----------|---------|--------|
| Code Structure | 8/10 | 9.5/10 |
| Error Handling | 8/10 | 9.5/10 |
| Code Duplication | 6.5/10 | 9/10 |
| Type Safety | 8/10 | 9.5/10 |
| Test Coverage | 7/10 | 9/10 |
| Performance | 7.5/10 | 9/10 |
| Security | 7.5/10 | 9.5/10 |
| Documentation | 8.5/10 | 9.5/10 |
| Dependencies | 8/10 | 9/10 |
| **Overall** | **7.9/10** | **9.3/10** |

---

## Notes

- All changes should maintain backward compatibility
- Run full test suite after each major change
- Update CHANGELOG.md for each completed section
- Consider creating feature branches for larger changes (Week 2+)
