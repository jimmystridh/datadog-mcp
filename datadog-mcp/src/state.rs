use anyhow::Result;
use datadog_api::{apis::MonitorsApi, DatadogClient, DatadogConfig};
use std::sync::Arc;

pub struct ServerState {
    pub client: Arc<DatadogClient>,
    pub config: DatadogConfig,
}

impl ServerState {
    pub async fn new(config: DatadogConfig) -> Result<Self> {
        let client = DatadogClient::new(config.clone())?;
        Ok(Self {
            client: Arc::new(client),
            config,
        })
    }

    pub async fn test_connection(&self) -> Result<()> {
        // Test with a simple API call (list monitors with minimal results)
        let api = MonitorsApi::new((*self.client).clone());
        api.list_monitors_with_page_size(1).await?;
        Ok(())
    }
}
