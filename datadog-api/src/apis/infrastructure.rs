use crate::{
    client::DatadogClient,
    models::{HostsResponse, TagsResponse},
    Result,
};
use serde::Serialize;

/// API client for Datadog infrastructure endpoints.
pub struct InfrastructureApi {
    client: DatadogClient,
}

impl InfrastructureApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_hosts(&self) -> Result<HostsResponse> {
        self.client.get("/api/v1/hosts").await
    }

    pub async fn get_tags(&self, source: Option<&str>) -> Result<TagsResponse> {
        if let Some(source) = source {
            #[derive(Serialize)]
            struct QueryParams<'a> {
                source: &'a str,
            }

            let params = QueryParams { source };

            self.client
                .get_with_query("/api/v1/tags/hosts", &params)
                .await
        } else {
            self.client.get("/api/v1/tags/hosts").await
        }
    }
}
