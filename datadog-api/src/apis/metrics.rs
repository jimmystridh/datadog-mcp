use crate::{
    client::DatadogClient,
    models::{MetricMetadata, MetricsListResponse, MetricsQueryResponse},
    Result,
};
use serde::Serialize;

pub struct MetricsApi {
    client: DatadogClient,
}

impl MetricsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

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

        self.client
            .get_with_query("/api/v1/query", &params)
            .await
    }

    pub async fn list_metrics(&self, query: &str) -> Result<MetricsListResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            q: &'a str,
        }

        let params = QueryParams { q: query };

        self.client
            .get_with_query("/api/v1/metrics", &params)
            .await
    }

    pub async fn get_metric_metadata(&self, metric_name: &str) -> Result<MetricMetadata> {
        let endpoint = format!("/api/v1/metrics/{}", metric_name);
        self.client.get(&endpoint).await
    }
}
