use anyhow::Result;
use datadog_api::{apis::MonitorsApi, DatadogClient, DatadogConfig};
use std::sync::Arc;

use crate::output::OutputFormat;

pub struct ServerState {
    pub client: Arc<DatadogClient>,
    pub config: DatadogConfig,
    pub output_format: OutputFormat,
}

/// Context passed to tool functions containing client and output format
#[derive(Clone)]
pub struct ToolContext {
    pub client: Arc<DatadogClient>,
    pub output_format: OutputFormat,
}

impl ToolContext {
    pub fn new(client: Arc<DatadogClient>, output_format: OutputFormat) -> Self {
        Self {
            client,
            output_format,
        }
    }
}

impl ServerState {
    pub async fn new(config: DatadogConfig, output_format: OutputFormat) -> Result<Self> {
        let client = DatadogClient::new(config.clone())?;
        Ok(Self {
            client: Arc::new(client),
            config,
            output_format,
        })
    }

    pub fn tool_context(&self) -> ToolContext {
        ToolContext::new(self.client.clone(), self.output_format)
    }

    pub async fn test_connection(&self) -> Result<()> {
        // Test with a simple API call (list monitors with minimal results)
        let api = MonitorsApi::new((*self.client).clone());
        api.list_monitors_with_page_size(1).await?;
        Ok(())
    }
}
