use crate::{
    client::DatadogClient,
    models::{MetricMetadata, MetricsListResponse, MetricsQueryResponse},
    Result,
};
use serde::Serialize;

/// API client for Datadog metrics endpoints.
pub struct MetricsApi {
    client: DatadogClient,
}

impl MetricsApi {
    /// Creates a new metrics API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    /// Query time series data for metrics.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn query_metrics(
        &self,
        from: i64,
        to: i64,
        query: &str,
    ) -> Result<MetricsQueryResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            from: i64,
            to: i64,
            query: &'a str,
        }

        let params = QueryParams { from, to, query };

        self.client.get_with_query("/api/v1/query", &params).await
    }

    /// List active metrics matching a query.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn list_metrics(&self, query: &str) -> Result<MetricsListResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            q: &'a str,
        }

        let params = QueryParams { q: query };

        self.client.get_with_query("/api/v1/metrics", &params).await
    }

    /// Get metadata for a specific metric.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_metric_metadata(&self, metric_name: &str) -> Result<MetricMetadata> {
        let endpoint = format!("/api/v1/metrics/{metric_name}");
        self.client.get(&endpoint).await
    }
}
