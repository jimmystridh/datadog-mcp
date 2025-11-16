use crate::{client::DatadogClient, models::SLOsResponse, Result};

/// API client for Datadog slos endpoints.
pub struct SLOsApi {
    client: DatadogClient,
}

impl SLOsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_slos(&self) -> Result<SLOsResponse> {
        self.client.get("/api/v1/slo").await
    }
}
