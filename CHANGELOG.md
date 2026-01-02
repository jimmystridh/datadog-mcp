# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
