use crate::{
    client::DatadogClient,
    models::{
        ServiceDependencies, ServiceStats, Span, Trace, TraceQuery, TraceSearchResponse,
        TraceSubmitRequest, TraceSubmitResponse,
    },
    Result,
};
use serde::Serialize;

/// API client for Datadog APM/Traces endpoints.
///
/// Enables distributed tracing, service performance monitoring, and dependency mapping.
pub struct TracesApi {
    client: DatadogClient,
}

impl TracesApi {
    /// Creates a new traces API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    /// Submit traces to Datadog APM.
    ///
    /// # Arguments
    ///
    /// * `traces` - Array of traces (each trace is an array of spans)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use datadog_api::{DatadogClient, DatadogConfig, models::Span};
    /// use datadog_api::apis::TracesApi;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = DatadogConfig::from_env()?;
    /// let client = DatadogClient::new(config)?;
    /// let traces_api = TracesApi::new(client);
    ///
    /// let span = Span {
    ///     span_id: 12345,
    ///     trace_id: 67890,
    ///     parent_id: 0,
    ///     service: "my-service".to_string(),
    ///     resource: "GET /api/users".to_string(),
    ///     name: "web.request".to_string(),
    ///     start: 1234567890000000000,
    ///     duration: 15000000,
    ///     error: 0,
    ///     meta: Default::default(),
    ///     metrics: Default::default(),
    ///     span_type: Some("web".to_string()),
    /// };
    ///
    /// traces_api.send_traces(vec![vec![span]]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_traces(&self, traces: Vec<Vec<Span>>) -> Result<TraceSubmitResponse> {
        let request = TraceSubmitRequest { traces };
        self.client.post("/v0.4/traces", &request).await
    }

    /// Search for traces matching the given criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_traces(&self, query: &TraceQuery) -> Result<TraceSearchResponse> {
        self.client.get_with_query("/api/v2/traces", query).await
    }

    /// Get a specific trace by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_trace(&self, trace_id: &str) -> Result<Trace> {
        let endpoint = format!("/api/v2/traces/{trace_id}");
        self.client.get(&endpoint).await
    }

    /// Get performance statistics for a service.
    ///
    /// # Arguments
    ///
    /// * `service` - Service name
    /// * `start` - Start time (seconds since epoch)
    /// * `end` - End time (seconds since epoch)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_service_stats(
        &self,
        service: &str,
        start: i64,
        end: i64,
    ) -> Result<ServiceStats> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            service: &'a str,
            start: i64,
            end: i64,
        }

        let params = QueryParams {
            service,
            start,
            end,
        };

        self.client
            .get_with_query("/api/v2/apm/stats", &params)
            .await
    }

    /// Get service dependencies (which services call which).
    ///
    /// # Arguments
    ///
    /// * `service` - Service name
    /// * `start` - Start time (seconds since epoch)
    /// * `end` - End time (seconds since epoch)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_service_dependencies(
        &self,
        service: &str,
        start: i64,
        end: i64,
    ) -> Result<ServiceDependencies> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            service: &'a str,
            start: i64,
            end: i64,
        }

        let params = QueryParams {
            service,
            start,
            end,
        };

        self.client
            .get_with_query("/api/v2/apm/dependencies", &params)
            .await
    }

    /// List all services that have sent traces.
    ///
    /// # Arguments
    ///
    /// * `start` - Start time (seconds since epoch)
    /// * `end` - End time (seconds since epoch)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn list_services(&self, start: i64, end: i64) -> Result<Vec<String>> {
        #[derive(Serialize)]
        struct QueryParams {
            start: i64,
            end: i64,
        }

        let params = QueryParams { start, end };

        self.client
            .get_with_query("/api/v2/apm/services", &params)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatadogConfig;
    use std::collections::HashMap;

    #[test]
    fn test_traces_api_creation() {
        let config = DatadogConfig::new("test_key".to_string(), "test_app_key".to_string());
        let client = DatadogClient::new(config).unwrap();
        let _traces_api = TracesApi::new(client);
    }

    #[test]
    fn test_span_serialization() {
        let span = Span {
            span_id: 12345,
            trace_id: 67890,
            parent_id: 0,
            service: "test-service".to_string(),
            resource: "GET /test".to_string(),
            name: "web.request".to_string(),
            start: 1234567890000000000,
            duration: 15000000,
            error: 0,
            meta: HashMap::new(),
            metrics: HashMap::new(),
            span_type: Some("web".to_string()),
        };

        let json = serde_json::to_string(&span).unwrap();
        assert!(json.contains("test-service"));
        assert!(json.contains("12345"));
    }

    #[test]
    fn test_trace_query_serialization() {
        let query = TraceQuery {
            service: Some("my-service".to_string()),
            operation: Some("web.request".to_string()),
            resource: None,
            start: 1234567890,
            end: 1234567900,
            limit: Some(100),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("my-service"));
        assert!(json.contains("web.request"));
    }
}
