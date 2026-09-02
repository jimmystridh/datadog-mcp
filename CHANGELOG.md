# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Updated all Rust dependencies and GitHub Actions to their latest releases.
- Migrated to RMCP 3.1 and its current MCP protocol negotiation, content model, and generated server metadata.
- Migrated to keyring 4.1's default cross-platform credential store and removed native D-Bus build dependencies.
- Replaced the unmaintained `dotenv` crate with the maintained `dotenvy` fork.
- Declared Rust 1.88 as the minimum supported version and added a committed lockfile for reproducible application builds.
- Bumped the workspace to 0.2.0 for corrected API contracts and safer MCP defaults.
- Switched metric queries to `/api/v2/query/timeseries`, Teams to `/api/v2/team`, and Users to `/api/v2/users`.
- Replaced unsupported trace endpoints with indexed span search plus current APM service and dependency endpoints.
- Made the MCP server read-only by default; Datadog mutations now require `--allow-write`.

### Improved

- Reused RMCP's cached tool router and removed the redundant custom initialization handler.
- Fixed active metric discovery by sending Datadog's required `from` timestamp and filtering metric names locally.
- Accepted nullable metric units and empty scale factors returned by the Datadog timeseries API.
- Compiled credential-redaction expressions once using Regex 1.13's `regex!` macro.
- Marked authentication headers as sensitive and zeroized temporary credential payloads.
- Removed unused direct dependencies and replaced Chrono usage with standard or project-native timestamp APIs where formatting was unnecessary.
- Fixed tests that did not compile without the optional TOON feature and added minimal-feature validation to CI.
- Added MCP structured content, inline bounded results, cursor/offset pagination, and correct `isError` tool results.
- Made `get_monitor` return group states, transition timestamps, active matching downtimes, and complete monitor options by default, with filters for narrower responses.
- Made dashboard and Synthetics updates preserve fields unknown to this client.
- Made response caching opt-in, excluded sensitive domains, confined cache reads, and added retention/size limits.
- Added method-aware retries with jitter, rate-limit reset handling, total deadlines, and no automatic write retries.
- Bounded decoded HTTP response bodies before JSON deserialization, with a configurable 10 MiB default.
- Added dependency auditing, formatting, MSRV checks, Dependabot, tag/version release validation, and immutable GitHub Action pins.

## [0.1.0] - 2025-01-02

### Added

#### datadog-api Library
- Complete Rust client for Datadog API with 16 API modules:
  - Monitors, Dashboards, Metrics, Logs, Events
  - Synthetics, Downtimes, Incidents, SLOs
  - Infrastructure, Security, Notebooks
  - Teams, Users, Traces
- Automatic retry with exponential backoff for transient failures
- Client-side rate limiting with token bucket algorithm
- Conditional requests with ETag/If-Modified-Since support
- Type-safe ID newtypes (MonitorId, DashboardId, etc.)
- Type-safe timestamps (TimestampSecs, TimestampMillis, TimestampNanos)
- Typed widget definitions for dashboards
- Secure credential storage via system keyring (macOS Keychain, Windows Credential Manager, Secret Service)
- Support for all Datadog regions (US1, US3, US5, EU, AP1, US1-FED)
- Configurable HTTP connection pool settings
- Comprehensive error handling with `is_retryable()`, `is_not_found()`, etc.

#### datadog-mcp Server
- 35+ MCP tools for interacting with Datadog:
  - Metrics & monitoring: get_metrics, search_metrics, get_monitors, create_monitor, etc.
  - Dashboards: get_dashboards, create_dashboard, update_dashboard, etc.
  - Logs & events: search_logs, get_events
  - Synthetics: create_synthetics_test, trigger_synthetics_tests, etc.
  - Infrastructure: get_infrastructure, get_tags, get_kubernetes_deployments
  - Downtimes: create_downtime, cancel_downtime
  - And more: incidents, SLOs, notebooks, teams, users
- TOON output format for 30-60% token reduction vs JSON
- Local caching of API responses
- Input validation and sanitization
- Response size limits with warnings
- Comprehensive error codes for MCP protocol

### Security
- Credentials zeroed from memory on drop (zeroize)
- Cache files created with 0o600 permissions on Unix
- API keys redacted from logs
- Error responses sanitized to prevent credential leakage

### Testing
- 199 tests across both crates
- Property-based tests with proptest
- HTTP mock tests with wiremock for all API endpoints
- Criterion benchmarks for serialization performance

### Documentation
- Comprehensive README with quick start guide
- API library documentation with examples
- Security documentation (SECURITY.md)
- Architecture documentation in lib.rs

[0.1.0]: https://github.com/jimmystridh/datadog-mcp/releases/tag/v0.1.0
