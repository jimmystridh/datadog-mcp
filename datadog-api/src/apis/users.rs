use crate::{client::DatadogClient, models::UsersResponse, Result};

pub struct UsersApi {
    client: DatadogClient,
}

impl UsersApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_users(&self) -> Result<UsersResponse> {
        self.client.get("/api/v1/users").await
    }
}
