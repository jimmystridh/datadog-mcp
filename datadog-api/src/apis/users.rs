use crate::{client::DatadogClient, models::UsersResponse, Result};

/// API client for Datadog users endpoints.
pub struct UsersApi {
    client: DatadogClient,
}

impl UsersApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_users(&self) -> Result<UsersResponse> {
        self.client.get("/api/v1/users").await
    }
}
