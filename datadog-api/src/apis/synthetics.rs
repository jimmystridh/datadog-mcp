use crate::{client::DatadogClient, models::SyntheticsTestsResponse, Result};

pub struct SyntheticsApi {
    client: DatadogClient,
}

impl SyntheticsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_tests(&self) -> Result<SyntheticsTestsResponse> {
        self.client.get("/api/v1/synthetics/tests").await
    }
}
