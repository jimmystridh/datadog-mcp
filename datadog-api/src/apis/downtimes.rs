use crate::{
    client::DatadogClient,
    models::{Downtime, DowntimeCreateRequest},
    Result,
};

/// API client for Datadog downtimes endpoints.
pub struct DowntimesApi {
    client: DatadogClient,
}

impl DowntimesApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_downtimes(&self) -> Result<Vec<Downtime>> {
        self.client.get("/api/v1/downtime").await
    }

    pub async fn create_downtime(&self, downtime: &DowntimeCreateRequest) -> Result<Downtime> {
        self.client.post("/api/v1/downtime", downtime).await
    }
}
