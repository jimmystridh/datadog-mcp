use crate::{client::DatadogClient, models::TeamsResponse, Result};

/// API client for Datadog teams endpoints.
pub struct TeamsApi {
    client: DatadogClient,
}

impl TeamsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_teams(&self) -> Result<TeamsResponse> {
        self.client.get("/api/v2/teams").await
    }
}
