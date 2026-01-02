use crate::rate_limit::{RateLimitConfig, RateLimiter};
use crate::{config::DatadogConfig, error::Error, Result};
use reqwest::{header, Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, error, trace};

fn sanitize_log_message(message: &str) -> String {
    use regex::Regex;

    let key_pattern = r#"dd-api-key|dd-application-key|DD_API_KEY|DD_APP_KEY|api_key|app_key|apikey|appkey"#;

    let patterns = [
        // JSON style: "api_key": "value" or "api_key":"value"
        format!(r#"(?i)"({key_pattern})"\s*:\s*"([^"]*)""#),
        // Header/env style with quoted value: api_key: "value" or api_key="value"
        format!(r#"(?i)({key_pattern})\s*[:=]\s*"([^"]*)""#),
        // Header/env style with single-quoted value
        format!(r#"(?i)({key_pattern})\s*[:=]\s*'([^']*)'"#),
        // Header/env style with unquoted value
        format!(r#"(?i)({key_pattern})\s*[:=]\s*([^\s,}}"'\n]+)"#),
    ];

    let mut result = message.to_string();
    for pattern in patterns {
        if let Ok(re) = Regex::new(&pattern) {
            result = re.replace_all(&result, "\"$1\": \"[REDACTED]\"").to_string();
        }
    }
    result
}

/// HTTP client for interacting with the Datadog API.
///
/// Handles authentication, request building, and response parsing for all Datadog API endpoints.
/// Includes client-side rate limiting to prevent hitting Datadog's API limits.
#[derive(Clone)]
pub struct DatadogClient {
    client: ClientWithMiddleware,
    config: DatadogConfig,
    rate_limiter: RateLimiter,
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
    pub fn with_rate_limit(config: DatadogConfig, rate_limit_config: RateLimitConfig) -> Result<Self> {
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(
                Duration::from_millis(config.retry_config.initial_backoff_ms),
                Duration::from_millis(config.retry_config.max_backoff_ms),
            )
            .build_with_max_retries(config.retry_config.max_retries);

        let retry_middleware = RetryTransientMiddleware::new_with_policy(retry_policy);

        let base_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .gzip(true)
            .build()
            .map_err(Error::HttpError)?;

        let client = ClientBuilder::new(base_client)
            .with(retry_middleware)
            .build();

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

    /// Checks if an endpoint corresponds to an unstable operation.
    fn is_unstable_operation(&self, endpoint: &str) -> bool {
        self.config
            .unstable_operations
            .iter()
            .any(|op| endpoint.contains(op))
    }

    fn build_headers(&self, endpoint: Option<&str>) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::HeaderName::from_static("dd-api-key"),
            header::HeaderValue::from_str(self.config.api_key.expose())
                .map_err(|e| Error::ConfigError(format!("Invalid API key: {e}")))?,
        );

        headers.insert(
            header::HeaderName::from_static("dd-application-key"),
            header::HeaderValue::from_str(self.config.app_key.expose())
                .map_err(|e| Error::ConfigError(format!("Invalid app key: {e}")))?,
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("datadog-mcp/0.1.0"),
        );

        headers.insert(
            header::ACCEPT_ENCODING,
            header::HeaderValue::from_static("gzip"),
        );

        // Add unstable operation header if needed
        if let Some(endpoint) = endpoint {
            if self.is_unstable_operation(endpoint) {
                headers.insert(
                    header::HeaderName::from_static("dd-operation-unstable"),
                    header::HeaderValue::from_static("true"),
                );
            }
        }

        Ok(headers)
    }

    fn add_auth_headers(&self, builder: RequestBuilder, endpoint: &str) -> Result<RequestBuilder> {
        Ok(builder.headers(self.build_headers(Some(endpoint))?))
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            trace!("Successful response with status: {status}");
            response.json::<T>().await.map_err(Error::HttpError)
        } else {
            let status_code = status.as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            let sanitized_body = sanitize_log_message(&error_body);
            error!("API error: {status_code} - {sanitized_body}");

            Err(Error::ApiError {
                status: status_code,
                message: sanitized_body,
            })
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("GET {url}");

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;

        self.handle_response(response).await
    }

    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);

        let request = self.client.get(&url).query(query);
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;

        debug!("Response status: {}", response.status());
        self.handle_response(response).await
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("POST {url}");

        let json_body = serde_json::to_string(body).map_err(Error::JsonError)?;
        let request = self
            .client
            .post(&url)
            .body(json_body)
            .header(header::CONTENT_TYPE, "application/json");
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;
        self.handle_response(response).await
    }

    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("PUT {url}");

        let json_body = serde_json::to_string(body).map_err(Error::JsonError)?;
        let request = self
            .client
            .put(&url)
            .body(json_body)
            .header(header::CONTENT_TYPE, "application/json");
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;
        self.handle_response(response).await
    }

    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("DELETE {url}");

        let request = self.client.delete(&url);
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let status_code = status.as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            let sanitized_body = sanitize_log_message(&error_body);
            error!("API error: {} - {}", status_code, sanitized_body);

            Err(Error::ApiError {
                status: status_code,
                message: sanitized_body,
            })
        }
    }

    pub async fn delete_with_response<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        self.rate_limiter.acquire().await;

        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("DELETE {url}");

        let request = self.client.delete(&url);
        let request = self.add_auth_headers(request, endpoint)?;

        let response = request.send().await.map_err(Error::MiddlewareError)?;
        self.handle_response(response).await
    }

    /// Returns a reference to the rate limiter (for monitoring)
    #[must_use]
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }
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
}
