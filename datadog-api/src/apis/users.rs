use crate::{client::DatadogClient, models::UsersV2Response, NumberedPage, Result};

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

    pub async fn list_users(&self, page: NumberedPage) -> Result<UsersV2Response> {
        self.client.get_with_query("/api/v2/users", &page).await
    }
}
