use crate::{client::DatadogClient, models::SLOsResponse, Result};

pub struct SLOsApi {
    client: DatadogClient,
}

impl SLOsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_slos(&self) -> Result<SLOsResponse> {
        self.client.get("/api/v1/slo").await
    }
}
