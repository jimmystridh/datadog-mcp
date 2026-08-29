use crate::{
    client::DatadogClient,
    models::{MetricMetadata, MetricsListResponse, TimeseriesFormulaQueryResponse},
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
    ) -> Result<TimeseriesFormulaQueryResponse> {
        let request = serde_json::json!({
            "data": {
                "attributes": {
                    "formulas": [{"formula": "a"}],
                    "from": from.saturating_mul(1000),
                    "queries": [{
                        "data_source": "metrics",
                        "name": "a",
                        "query": query,
                    }],
                    "to": to.saturating_mul(1000),
                },
                "type": "timeseries_request",
            }
        });

        self.client
            .post_retryable("/api/v2/query/timeseries", &request)
            .await
    }

    /// List metrics that have reported data since the provided timestamp.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn list_active_metrics(&self, from: i64) -> Result<MetricsListResponse> {
        #[derive(Serialize)]
        struct QueryParams {
            from: i64,
        }

        let params = QueryParams { from };

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
