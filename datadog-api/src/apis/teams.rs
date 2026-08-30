use crate::{client::DatadogClient, models::TeamsResponse, NumberedPage, Result};

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

    pub async fn list_teams(&self, page: NumberedPage) -> Result<TeamsResponse> {
        self.client.get_with_query("/api/v2/team", &page).await
    }
}
