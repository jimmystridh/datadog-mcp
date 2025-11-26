use anyhow::Result;
use datadog_api::{
    apis::{
        DashboardsApi, DowntimesApi, EventsApi, IncidentsApi, InfrastructureApi, LogsApi,
        MetricsApi, MonitorsApi, NotebooksApi, SLOsApi, SecurityApi, SyntheticsApi, TeamsApi,
        UsersApi,
    },
    DatadogClient, DatadogConfig,
};
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

    pub fn metrics_api(&self) -> MetricsApi {
        MetricsApi::new((*self.client).clone())
    }

    pub fn monitors_api(&self) -> MonitorsApi {
        MonitorsApi::new((*self.client).clone())
    }

    pub fn dashboards_api(&self) -> DashboardsApi {
        DashboardsApi::new((*self.client).clone())
    }

    pub fn logs_api(&self) -> LogsApi {
        LogsApi::new((*self.client).clone())
    }

    pub fn events_api(&self) -> EventsApi {
        EventsApi::new((*self.client).clone())
    }

    pub fn infrastructure_api(&self) -> InfrastructureApi {
        InfrastructureApi::new((*self.client).clone())
    }

    pub fn downtimes_api(&self) -> DowntimesApi {
        DowntimesApi::new((*self.client).clone())
    }

    pub fn synthetics_api(&self) -> SyntheticsApi {
        SyntheticsApi::new((*self.client).clone())
    }

    pub fn security_api(&self) -> SecurityApi {
        SecurityApi::new((*self.client).clone())
    }

    pub fn incidents_api(&self) -> IncidentsApi {
        IncidentsApi::new((*self.client).clone())
    }

    pub fn slos_api(&self) -> SLOsApi {
        SLOsApi::new((*self.client).clone())
    }

    pub fn notebooks_api(&self) -> NotebooksApi {
        NotebooksApi::new((*self.client).clone())
    }

    pub fn teams_api(&self) -> TeamsApi {
        TeamsApi::new((*self.client).clone())
    }

    pub fn users_api(&self) -> UsersApi {
        UsersApi::new((*self.client).clone())
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
