use crate::{client::DatadogClient, models::NotebooksResponse, Result};

pub struct NotebooksApi {
    client: DatadogClient,
}

impl NotebooksApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_notebooks(&self) -> Result<NotebooksResponse> {
        self.client.get("/api/v1/notebooks").await
    }
}
