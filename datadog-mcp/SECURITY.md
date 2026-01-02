# Security Documentation

This document describes the security measures implemented in the Datadog MCP server and API client.

## Credential Handling

### Storage Priority

Credentials are loaded in this order (first available wins):

1. **Credentials file**: `~/.datadog-mcp/credentials.json`
2. **System keyring** (requires `keyring` feature): OS-native secure storage
3. **Environment variables**: `DD_API_KEY`, `DD_APP_KEY`, `DD_SITE`

### Secret Protection

- **Zeroize on drop**: API keys use `SecretString` wrapper with `zeroize` crate
- **Memory is overwritten** when credentials go out of scope
- **Debug/Display redaction**: Credentials print as `[REDACTED]` in logs

```rust
// Credentials are never exposed in debug output
let config = DatadogConfig::from_env()?;
println!("{:?}", config);  // Shows "[REDACTED]" for keys
```

### File Permissions

- Cache files are created with **0600 permissions** (owner read/write only)
- Credentials file should be protected: `chmod 600 ~/.datadog-mcp/credentials.json`

## Network Security

### TLS/HTTPS

- All API requests use HTTPS exclusively
- TLS certificate verification is enabled (system trust store)
- No support for disabling certificate verification

### Request Authentication

- API keys sent via `DD-API-KEY` and `DD-APPLICATION-KEY` headers
- Never included in URLs or query parameters
- Headers are sanitized before logging

## Logging and Caching

### Sanitized Logging

Error messages and logs are sanitized to remove credential patterns:

- `DD_API_KEY=...` patterns → `[REDACTED]`
- `api_key: "..."` patterns → `[REDACTED]`
- Header values for auth headers → `[REDACTED]`

```rust
// Input: "dd-api-key: secret123abc"
// Log output: "dd-api-key: [REDACTED]"
```

### Cache Security

- Cache directory: `~/.cache/datadog-mcp/` (XDG compliant)
- Files created with restrictive permissions (Unix 0600)
- Contains API response data, not credentials
- Automatic cleanup available via `cleanup_cache` tool

## Rate Limiting

Client-side rate limiting prevents API abuse:

- Default: 10 requests/second with 2-second burst allowance
- Token bucket algorithm prevents accidental DoS
- Configurable via `RateLimitConfig`

## Input Validation

### Monitor Types

Only valid Datadog monitor types are accepted:
- `metric alert`, `log alert`, `query alert`, `event alert`
- `service check`, `process alert`, `synthetics alert`
- And others per Datadog API specification

### Dashboard Layouts

Only valid layouts: `ordered` or `free`

### Input Sanitization

- Tag values: sanitized to alphanumeric + allowed special chars
- Query strings: length-limited, special chars escaped
- Names: truncated to reasonable lengths

## Threat Model

### In Scope

| Threat | Mitigation |
|--------|------------|
| Credential exposure in logs | Sanitization + redaction |
| Credential exposure in memory | Zeroize on drop |
| Unauthorized file access | 0600 permissions |
| Rate limit abuse | Client-side limiting |
| Invalid API input | Type-safe validation |

### Out of Scope

| Threat | Reason |
|--------|--------|
| Compromised host | Assumes trusted execution environment |
| Memory dumps | Zeroize helps but not foolproof |
| Network interception | TLS required, assumes valid certificates |

## Configuration

### Disable Features

Build without optional security features:

```bash
# Without keyring support (no system credential storage)
cargo build --no-default-features --features toon

# Minimal build
cargo build --no-default-features
```

### Environment Variables

| Variable | Purpose | Security Note |
|----------|---------|---------------|
| `DD_API_KEY` | Datadog API key | Use file/keyring in production |
| `DD_APP_KEY` | Datadog app key | Use file/keyring in production |
| `DD_SITE` | Datadog site | No secret data |
| `DD_PROFILE` | Keyring profile | No secret data |
| `DATADOG_MCP_CACHE_DIR` | Cache location | Ensure secure permissions |

## Reporting Security Issues

If you discover a security vulnerability, please report it via:
- GitHub Security Advisories (private disclosure)
- Do not open public issues for security vulnerabilities
