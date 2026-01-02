use crate::{client::DatadogClient, models::NotebooksResponse, Result};

/// API client for Datadog notebooks endpoints.
pub struct NotebooksApi {
    client: DatadogClient,
}

impl NotebooksApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_notebooks(&self) -> Result<NotebooksResponse> {
        self.client.get("/api/v1/notebooks").await
    }
}
