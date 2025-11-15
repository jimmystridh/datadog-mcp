use crate::{client::DatadogClient, models::TeamsResponse, Result};

pub struct TeamsApi {
    client: DatadogClient,
}

impl TeamsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_teams(&self) -> Result<TeamsResponse> {
        self.client.get("/api/v2/teams").await
    }
}
