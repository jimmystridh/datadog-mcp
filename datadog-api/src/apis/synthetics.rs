use crate::{client::DatadogClient, models::SyntheticsTestsResponse, Result};

/// API client for Datadog synthetics endpoints.
pub struct SyntheticsApi {
    client: DatadogClient,
}

impl SyntheticsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_tests(&self) -> Result<SyntheticsTestsResponse> {
        self.client.get("/api/v1/synthetics/tests").await
    }
}
