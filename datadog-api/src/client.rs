use crate::rate_limit::{RateLimitConfig, RateLimiter};
use crate::{
    config::{DatadogConfig, RetryConfig},
    error::Error,
    Result,
};
use futures_util::StreamExt;
use reqwest::{header, Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::Instant;
use tracing::{debug, error, trace, warn};

fn sanitize_log_message(message: &str) -> String {
    use regex::regex;

    let patterns = [
        regex!(
            r#"(?i)"(dd-api-key|dd-application-key|DD_API_KEY|DD_APP_KEY|api_key|app_key|apikey|appkey)"\s*:\s*"([^"]*)""#
        ),
        regex!(
            r#"(?i)(dd-api-key|dd-application-key|DD_API_KEY|DD_APP_KEY|api_key|app_key|apikey|appkey)\s*[:=]\s*"([^"]*)""#
        ),
        regex!(
            r#"(?i)(dd-api-key|dd-application-key|DD_API_KEY|DD_APP_KEY|api_key|app_key|apikey|appkey)\s*[:=]\s*'([^']*)'"#
        ),
        regex!(
            r#"(?i)(dd-api-key|dd-application-key|DD_API_KEY|DD_APP_KEY|api_key|app_key|apikey|appkey)\s*[:=]\s*([^\s,}}"'\n]+)"#
        ),
    ];

    let mut result = message.to_string();
    for pattern in &patterns {
        result = pattern
            .replace_all(&result, "\"$1\": \"[REDACTED]\"")
            .to_string();
    }
    result
}

/// HTTP client for interacting with the Datadog API.
///
/// Handles authentication, request building, and response parsing for all Datadog API endpoints.
/// Includes client-side rate limiting to prevent hitting Datadog's API limits.
#[derive(Clone)]
pub struct DatadogClient {
    client: Client,
    config: DatadogConfig,
    rate_limiter: RateLimiter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetryClass {
    Safe,
    Never,
}

struct RetryPolicy<'a> {
    config: &'a RetryConfig,
}

impl RetryPolicy<'_> {
    fn permits_retry(&self, class: RetryClass, attempt: u32) -> bool {
        class == RetryClass::Safe && attempt < self.config.max_retries
    }

    fn backoff(&self, attempt: u32) -> Duration {
        let multiplier = self
            .config
            .backoff_multiplier
            .powi(i32::try_from(attempt).unwrap_or(i32::MAX));
        let base_ms = (self.config.initial_backoff_ms as f64 * multiplier)
            .min(self.config.max_backoff_ms as f64) as u64;
        let jitter_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        let jitter_ms = if base_ms == 0 {
            0
        } else {
            jitter_seed % (base_ms / 4 + 1)
        };
        Duration::from_millis(base_ms.saturating_add(jitter_ms))
    }

    fn response_backoff(&self, response: &Response, attempt: u32) -> Duration {
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            response
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or_else(|| self.backoff(attempt))
        } else {
            self.backoff(attempt)
        }
    }
}

struct PendingResponse {
    response: Response,
    deadline: Instant,
}

impl DatadogClient {
    /// Creates a new Datadog API client with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(config: DatadogConfig) -> Result<Self> {
        Self::with_rate_limit(config, RateLimitConfig::default())
    }

    /// Creates a new Datadog API client with custom rate limiting configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn with_rate_limit(
        config: DatadogConfig,
        rate_limit_config: RateLimitConfig,
    ) -> Result<Self> {
        config.validate_site()?;
        if rate_limit_config.enabled && rate_limit_config.requests_per_second == 0 {
            return Err(Error::ConfigError(
                "Rate limit must be greater than zero when enabled".to_string(),
            ));
        }

        let default_headers = Self::default_headers(&config)?;
        let http_config = &config.http_config;
        let mut builder = Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(http_config.timeout_secs))
            .pool_max_idle_per_host(http_config.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(http_config.pool_idle_timeout_secs))
            .gzip(true);

        if let Some(keepalive_secs) = http_config.tcp_keepalive_secs {
            builder = builder.tcp_keepalive(Duration::from_secs(keepalive_secs));
        }

        let client = builder.build().map_err(Error::HttpError)?;

        let rate_limiter = RateLimiter::new(rate_limit_config);

        Ok(Self {
            client,
            config,
            rate_limiter,
        })
    }

    /// Returns a reference to the configuration used by this client.
    #[must_use]
    pub fn config(&self) -> &DatadogConfig {
        &self.config
    }

    /// Validate both API and application keys with Datadog's dedicated endpoint.
    pub async fn validate_keys(&self) -> Result<serde_json::Value> {
        self.get("/api/v2/validate_keys").await
    }

    /// Checks if an endpoint corresponds to an unstable operation.
    fn is_unstable_operation(&self, endpoint: &str) -> bool {
        self.config
            .unstable_operations
            .iter()
            .any(|op| endpoint.contains(op))
    }

    fn default_headers(config: &DatadogConfig) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();
        let mut api_key = header::HeaderValue::from_str(config.api_key.expose())
            .map_err(|e| Error::ConfigError(format!("Invalid API key: {e}")))?;
        api_key.set_sensitive(true);
        let mut app_key = header::HeaderValue::from_str(config.app_key.expose())
            .map_err(|e| Error::ConfigError(format!("Invalid app key: {e}")))?;
        app_key.set_sensitive(true);

        headers.insert(header::HeaderName::from_static("dd-api-key"), api_key);
        headers.insert(
            header::HeaderName::from_static("dd-application-key"),
            app_key,
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            )),
        );

        Ok(headers)
    }

    fn add_operation_headers(&self, builder: RequestBuilder, endpoint: &str) -> RequestBuilder {
        if self.is_unstable_operation(endpoint) {
            builder.header("dd-operation-unstable", "true")
        } else {
            builder
        }
    }

    async fn send_with_policy<F>(
        &self,
        method: &'static str,
        endpoint: &str,
        retry_class: RetryClass,
        build: F,
    ) -> Result<PendingResponse>
    where
        F: Fn() -> RequestBuilder,
    {
        let started = Instant::now();
        let deadline = started + Duration::from_secs(self.config.http_config.total_timeout_secs);
        let policy = RetryPolicy {
            config: &self.config.retry_config,
        };
        let mut attempt = 0;

        loop {
            let response = tokio::time::timeout_at(deadline, async {
                self.rate_limiter.acquire().await;
                build().send().await
            })
            .await
            .map_err(|_| self.deadline_error())?;

            match response {
                Ok(response) => {
                    let status = response.status();
                    if !Error::is_retryable_status(status.as_u16())
                        || !policy.permits_retry(retry_class, attempt)
                    {
                        debug!(
                            method,
                            endpoint,
                            status = status.as_u16(),
                            retries = attempt,
                            duration_ms = started.elapsed().as_millis(),
                            "Datadog API request completed"
                        );
                        return Ok(PendingResponse { response, deadline });
                    }

                    let backoff = policy.response_backoff(&response, attempt);
                    warn!(
                        method,
                        endpoint,
                        status = status.as_u16(),
                        attempt = attempt + 1,
                        backoff_ms = backoff.as_millis(),
                        "Retrying transient Datadog API response"
                    );
                    self.sleep_before_deadline(deadline, backoff).await?;
                }
                Err(error) => {
                    let retryable = error.is_connect() || error.is_timeout();
                    if !retryable || !policy.permits_retry(retry_class, attempt) {
                        return Err(Error::HttpError(error));
                    }
                    let backoff = policy.backoff(attempt);
                    warn!(
                        method,
                        endpoint,
                        attempt = attempt + 1,
                        backoff_ms = backoff.as_millis(),
                        "Retrying transient Datadog transport error"
                    );
                    self.sleep_before_deadline(deadline, backoff).await?;
                }
            }

            attempt += 1;
        }
    }

    fn deadline_error(&self) -> Error {
        Error::RequestDeadlineExceeded(self.config.http_config.total_timeout_secs)
    }

    async fn sleep_before_deadline(&self, deadline: Instant, duration: Duration) -> Result<()> {
        let wake_at = Instant::now()
            .checked_add(duration)
            .ok_or_else(|| self.deadline_error())?;
        if wake_at >= deadline {
            return Err(self.deadline_error());
        }
        tokio::time::sleep_until(wake_at).await;
        Ok(())
    }

    async fn read_response_body(&self, response: Response) -> Result<Vec<u8>> {
        let limit = self.config.http_config.max_response_bytes;
        if let Some(length) = response.content_length() {
            let length = usize::try_from(length).unwrap_or(usize::MAX);
            if length > limit {
                return Err(Error::ResponseTooLarge {
                    size: length,
                    limit,
                });
            }
        }

        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(Error::HttpError)?;
            let size = body.len().saturating_add(chunk.len());
            if size > limit {
                return Err(Error::ResponseTooLarge { size, limit });
            }
            body.extend_from_slice(&chunk);
        }
        Ok(body)
    }

    async fn api_error(response: Response) -> Error {
        const MAX_ERROR_BODY_BYTES: usize = 4096;

        let status = response.status().as_u16();
        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while body.len() < MAX_ERROR_BODY_BYTES {
            match stream.next().await {
                Some(Ok(chunk)) => {
                    let remaining = MAX_ERROR_BODY_BYTES - body.len();
                    body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
                }
                Some(Err(error)) => {
                    debug!("Failed to read error response body: {error}");
                    break;
                }
                None => break,
            }
        }

        let message = String::from_utf8_lossy(&body);
        let sanitized_body = sanitize_log_message(&message);
        error!("API error: {status} - {sanitized_body}");
        Error::ApiError {
            status,
            message: sanitized_body,
        }
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            trace!("Successful response with status: {status}");
            let body = self.read_response_body(response).await?;
            serde_json::from_slice(&body).map_err(Error::JsonError)
        } else {
            Err(Self::api_error(response).await)
        }
    }

    async fn handle_pending_response<T: DeserializeOwned>(
        &self,
        pending: PendingResponse,
    ) -> Result<T> {
        tokio::time::timeout_at(pending.deadline, self.handle_response(pending.response))
            .await
            .map_err(|_| self.deadline_error())?
    }

    async fn ensure_pending_success(&self, pending: PendingResponse) -> Result<()> {
        tokio::time::timeout_at(pending.deadline, async {
            if pending.response.status().is_success() {
                Ok(())
            } else {
                Err(Self::api_error(pending.response).await)
            }
        })
        .await
        .map_err(|_| self.deadline_error())?
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let pending = self
            .send_with_policy("GET", endpoint, RetryClass::Safe, || {
                self.add_operation_headers(self.client.get(&url), endpoint)
            })
            .await?;

        self.handle_pending_response(pending).await
    }

    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let pending = self
            .send_with_policy("GET", endpoint, RetryClass::Safe, || {
                self.add_operation_headers(self.client.get(&url).query(query), endpoint)
            })
            .await?;
        self.handle_pending_response(pending).await
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let json_body = serde_json::to_string(body).map_err(Error::JsonError)?;
        let pending = self
            .send_with_policy("POST", endpoint, RetryClass::Never, || {
                self.add_operation_headers(
                    self.client
                        .post(&url)
                        .body(json_body.clone())
                        .header(header::CONTENT_TYPE, "application/json"),
                    endpoint,
                )
            })
            .await?;
        self.handle_pending_response(pending).await
    }

    /// POST a read-only search request with transient retries.
    pub async fn post_search<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let json_body = serde_json::to_string(body).map_err(Error::JsonError)?;
        let pending = self
            .send_with_policy("POST", endpoint, RetryClass::Safe, || {
                self.add_operation_headers(
                    self.client
                        .post(&url)
                        .body(json_body.clone())
                        .header(header::CONTENT_TYPE, "application/json"),
                    endpoint,
                )
            })
            .await?;
        self.handle_pending_response(pending).await
    }

    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let json_body = serde_json::to_string(body).map_err(Error::JsonError)?;
        let pending = self
            .send_with_policy("PUT", endpoint, RetryClass::Never, || {
                self.add_operation_headers(
                    self.client
                        .put(&url)
                        .body(json_body.clone())
                        .header(header::CONTENT_TYPE, "application/json"),
                    endpoint,
                )
            })
            .await?;
        self.handle_pending_response(pending).await
    }

    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let pending = self
            .send_with_policy("DELETE", endpoint, RetryClass::Never, || {
                self.add_operation_headers(self.client.delete(&url), endpoint)
            })
            .await?;
        self.ensure_pending_success(pending).await
    }

    pub async fn delete_with_response<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let pending = self
            .send_with_policy("DELETE", endpoint, RetryClass::Never, || {
                self.add_operation_headers(self.client.delete(&url), endpoint)
            })
            .await?;
        self.handle_pending_response(pending).await
    }

    /// Returns a reference to the rate limiter (for monitoring)
    #[must_use]
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// GET request with conditional caching support (ETag/Last-Modified).
    ///
    /// Returns `Ok(Some(response))` if data was modified, or `Ok(None)` if
    /// the cached version is still valid (304 Not Modified).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - API endpoint
    /// * `cache_info` - Optional cache info from a previous response
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use datadog_api::{DatadogClient, DatadogConfig, CachedResponse};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DatadogClient::new(DatadogConfig::from_env()?)?;
    ///
    /// // First request - no cache
    /// let response: CachedResponse<serde_json::Value> = client
    ///     .get_cached("/api/v1/monitor", None)
    ///     .await?
    ///     .expect("First request should return data");
    ///
    /// // Subsequent request with cache info
    /// match client.get_cached::<serde_json::Value>("/api/v1/monitor", Some(&response.cache_info)).await? {
    ///     Some(new_response) => println!("Data was modified"),
    ///     None => println!("Data unchanged, use cached version"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_cached<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        cache_info: Option<&CacheInfo>,
    ) -> Result<Option<CachedResponse<T>>> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        let pending = self
            .send_with_policy("GET", endpoint, RetryClass::Safe, || {
                let mut request = self.client.get(&url);
                if let Some(info) = cache_info {
                    if let Some(etag) = &info.etag {
                        request = request.header(header::IF_NONE_MATCH, etag.as_str());
                    }
                    if let Some(last_modified) = &info.last_modified {
                        request = request.header(header::IF_MODIFIED_SINCE, last_modified.as_str());
                    }
                }
                self.add_operation_headers(request, endpoint)
            })
            .await?;

        // 304 Not Modified - cached data is still valid
        if pending.response.status() == reqwest::StatusCode::NOT_MODIFIED {
            debug!("304 Not Modified - using cached data");
            return Ok(None);
        }

        // Extract cache headers before consuming the response
        let new_cache_info = CacheInfo {
            etag: pending
                .response
                .headers()
                .get(header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            last_modified: pending
                .response
                .headers()
                .get(header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        };

        let data: T = self.handle_pending_response(pending).await?;

        Ok(Some(CachedResponse {
            data,
            cache_info: new_cache_info,
        }))
    }
}

/// Cache validation information from HTTP headers
#[derive(Debug, Clone, Default)]
pub struct CacheInfo {
    /// ETag header value for conditional requests
    pub etag: Option<String>,
    /// Last-Modified header value for conditional requests
    pub last_modified: Option<String>,
}

impl CacheInfo {
    /// Check if any cache validation info is available
    #[must_use]
    pub fn has_validators(&self) -> bool {
        self.etag.is_some() || self.last_modified.is_some()
    }
}

/// Response with cache validation information
#[derive(Debug, Clone)]
pub struct CachedResponse<T> {
    /// The response data
    pub data: T,
    /// Cache information for subsequent conditional requests
    pub cache_info: CacheInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_json_api_key() {
        let input = r#"{"error": "Invalid api_key: abc123secret"}"#;
        let output = sanitize_log_message(input);
        assert!(!output.contains("abc123secret"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_header_style() {
        let input = "dd-api-key: secret123abc";
        let output = sanitize_log_message(input);
        assert!(!output.contains("secret123abc"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_env_var_style() {
        let input = "DD_API_KEY=mysecretkey and DD_APP_KEY=anothersecret";
        let output = sanitize_log_message(input);
        assert!(!output.contains("mysecretkey"));
        assert!(!output.contains("anothersecret"));
    }

    #[test]
    fn test_sanitize_quoted_value() {
        let input = r#"{"api_key": "secret_value_here", "other": "data"}"#;
        let output = sanitize_log_message(input);
        assert!(!output.contains("secret_value_here"));
        assert!(output.contains("other"));
    }

    #[test]
    fn test_sanitize_no_secrets() {
        let input = "This is a normal error message without any secrets";
        let output = sanitize_log_message(input);
        assert_eq!(input, output);
    }

    #[test]
    fn test_sanitize_case_insensitive() {
        let input = "API_KEY=secret123";
        let output = sanitize_log_message(input);
        assert!(!output.contains("secret123"));
    }

    #[test]
    fn test_default_headers_mark_credentials_sensitive() {
        let config = DatadogConfig::new("test_api_key".into(), "test_app_key".into());
        let headers = DatadogClient::default_headers(&config).unwrap();

        assert!(headers["dd-api-key"].is_sensitive());
        assert!(headers["dd-application-key"].is_sensitive());
        assert_eq!(
            headers[header::USER_AGENT],
            concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn test_cache_info_default() {
        let info = CacheInfo::default();
        assert!(info.etag.is_none());
        assert!(info.last_modified.is_none());
        assert!(!info.has_validators());
    }

    #[test]
    fn test_cache_info_with_etag() {
        let info = CacheInfo {
            etag: Some("\"abc123\"".to_string()),
            last_modified: None,
        };
        assert!(info.has_validators());
    }

    #[test]
    fn test_cache_info_with_last_modified() {
        let info = CacheInfo {
            etag: None,
            last_modified: Some("Wed, 21 Oct 2025 07:28:00 GMT".to_string()),
        };
        assert!(info.has_validators());
    }

    #[test]
    fn test_cached_response() {
        let response = CachedResponse {
            data: vec![1, 2, 3],
            cache_info: CacheInfo {
                etag: Some("\"test-etag\"".to_string()),
                last_modified: Some("Wed, 21 Oct 2025 07:28:00 GMT".to_string()),
            },
        };
        assert_eq!(response.data, vec![1, 2, 3]);
        assert!(response.cache_info.has_validators());
    }
}
