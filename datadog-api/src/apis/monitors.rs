use crate::{
    client::DatadogClient,
    models::{Monitor, MonitorCreateRequest, MonitorUpdateRequest},
    Result,
};
use serde::Serialize;

/// API client for Datadog monitors endpoints.
pub struct MonitorsApi {
    client: DatadogClient,
}

impl MonitorsApi {
    /// Creates a new monitors API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_monitors(&self) -> Result<Vec<Monitor>> {
        self.client.get("/api/v1/monitor").await
    }

    pub async fn list_monitors_with_page_size(&self, page_size: i32) -> Result<Vec<Monitor>> {
        #[derive(Serialize)]
        struct QueryParams {
            page_size: i32,
        }

        let params = QueryParams { page_size };

        self.client
            .get_with_query("/api/v1/monitor", &params)
            .await
    }

    pub async fn get_monitor(&self, monitor_id: i64) -> Result<Monitor> {
        let endpoint = format!("/api/v1/monitor/{}", monitor_id);
        self.client.get(&endpoint).await
    }

    pub async fn create_monitor(&self, monitor: &MonitorCreateRequest) -> Result<Monitor> {
        self.client.post("/api/v1/monitor", monitor).await
    }

    pub async fn update_monitor(
        &self,
        monitor_id: i64,
        update: &MonitorUpdateRequest,
    ) -> Result<Monitor> {
        let endpoint = format!("/api/v1/monitor/{}", monitor_id);
        self.client.put(&endpoint, update).await
    }

    pub async fn delete_monitor(&self, monitor_id: i64) -> Result<()> {
        let endpoint = format!("/api/v1/monitor/{}", monitor_id);
        self.client.delete(&endpoint).await
    }
}
