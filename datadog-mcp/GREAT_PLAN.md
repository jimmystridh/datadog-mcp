# GREAT PLAN — Datadog MCP Improvements

## Goals
- Reduce tool boilerplate and clarify module ownership.
- Improve type safety and error fidelity for MCP responses.
- Strengthen security posture and runtime robustness.
- Raise test coverage with HTTP mocks and tool-level integration checks.
- Document architecture and usage patterns for contributors and users.

## Workstreams & Key Tasks

### 1) Tools structure & duplication (P0–P1)
- Adopt the existing `tool_response!` helpers everywhere; remove manual `match Ok/Err` blocks in `src/tools.rs` and `src/tools_part2.rs`.
- Split tool implementations into domain modules under `src/tools/` with `mod.rs` re-exports (metrics, monitors, dashboards, logs, events, infra, synthetics, downtime, security, incidents, slos, notebooks, teams, users, cache_tools, validation).
- Move tool registration glue out of `src/server.rs` into a `tools_registry` module to keep server init lean.
- Add API constructor helpers on `ToolContext` (e.g., `metrics_api()`, `monitors_api()`) to avoid repeated `FooApi::new((*ctx.client).clone())`.

### 2) Type safety (P1–P2)
- Introduce newtype IDs (e.g., `DashboardId`, `MonitorId`, `DowntimeId`, `SyntheticsTestId`) and use them in tool inputs and API surfaces.
- Replace `serde_json::Value` options in `tool_inputs.rs` with typed option structs (e.g., `MonitorOptions`, `DashboardWidget` shapes as available).
- Add a timestamp newtype (seconds) to make units explicit for from/to parameters.

### 3) Errors & logging (P1–P2)
- Add an MCP error code map (parse/invalid params/not found/api/rate-limit/config/network) and return structured `ErrorData` instead of blanket `-32603`.
- Sanitize error strings and request logs to avoid leaking URLs or secrets; redact keys in traces.
- Wrap tool handlers in tracing spans with operation, inputs (redacted), duration, and outcome.

### 4) Security hardening (P0–P2)
- Zeroize API/app keys with a `SecretString` wrapper.
- Keep cache file permissions locked to `0600` (already present); add a sanity check in cache init if needed.
- Avoid overriding existing env vars with `.env` unless explicitly requested (make behavior opt-in).
- Add a lightweight client-side rate limiter/backoff guard to avoid bursts against Datadog.

### 5) Performance & robustness (P1–P2)
- Enable gzip/deflate on the reqwest client; tune Tokio features to the minimal set in `Cargo.toml`.
- Add pagination helpers and conditional requests (ETag/Last-Modified) where APIs support it.
- Make connection pool/timeout configurable via `DatadogConfig`.

### 6) Testing (P0–P2)
- Add `wiremock`-based HTTP tests for representative endpoints (metrics, monitors, dashboards, logs, synthetics) covering 200/401/403/404/429/500.
- Add tool integration tests to validate MCP tool outputs (status, summary, filepath presence).
- Use `serial_test` for env-var-sensitive tests; add a few property-based round-trips for models.
- Target: increase coverage notably over current (aim for ~9/10 qualitative score).

### 7) Documentation & examples (P2)
- Add `docs/ARCHITECTURE.md` (request flow, cache, error strategy) and `docs/SECURITY.md`.
- Provide `examples/` (basic usage, error handling, standalone API, batch patterns).
- Update README/CHANGELOG after major workstreams land.

## Suggested Timeline (flexible)
- **Week 1 (quick wins):** Tool response macro adoption; cache/env hardening tweak; reqwest compression; tokio feature trim.
- **Week 2 (structure):** Domain split for tools; ToolContext API helpers; server registry module; start wiremock tests.
- **Week 3 (type & error):** Newtype IDs, typed inputs, error code map/log sanitization; expand HTTP + tool integration tests.
- **Week 4 (polish):** Pagination/conditional requests, rate-limit guard, property-based tests, docs/examples refresh.

## Validation & Rollout
- CI: `cargo fmt`, `cargo clippy --workspace --all-targets --all-features`, `cargo test --workspace`.
- Add a mock-based test suite gate; keep live-Datadog tests optional/behind a feature if ever added.
- Manual smoke: run `datadog-mcp --format toon` with dummy/mocked config to ensure startup and tool listing still work.
