use crate::{client::DatadogClient, models::IncidentsResponse, OffsetPage, Result};

/// API client for Datadog incidents endpoints.
pub struct IncidentsApi {
    client: DatadogClient,
}

impl IncidentsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_incidents(&self, page: OffsetPage) -> Result<IncidentsResponse> {
        self.client.get_with_query("/api/v2/incidents", &page).await
    }
}
