use crate::{
    client::DatadogClient,
    models::{Dashboard, DashboardListResponse},
    Result,
};

pub struct DashboardsApi {
    client: DatadogClient,
}

impl DashboardsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_dashboards(&self) -> Result<DashboardListResponse> {
        self.client.get("/api/v1/dashboard").await
    }

    pub async fn get_dashboard(&self, dashboard_id: &str) -> Result<Dashboard> {
        let endpoint = format!("/api/v1/dashboard/{}", dashboard_id);
        self.client.get(&endpoint).await
    }

    pub async fn create_dashboard(&self, dashboard: &Dashboard) -> Result<Dashboard> {
        self.client.post("/api/v1/dashboard", dashboard).await
    }

    pub async fn update_dashboard(
        &self,
        dashboard_id: &str,
        dashboard: &Dashboard,
    ) -> Result<Dashboard> {
        let endpoint = format!("/api/v1/dashboard/{}", dashboard_id);
        self.client.put(&endpoint, dashboard).await
    }

    pub async fn delete_dashboard(&self, dashboard_id: &str) -> Result<()> {
        let endpoint = format!("/api/v1/dashboard/{}", dashboard_id);
        self.client.delete(&endpoint).await
    }
}
