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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> DatadogConfig {
        DatadogConfig::new("test_api_key".into(), "test_app_key".into())
    }

    #[tokio::test]
    async fn test_server_state_creation() {
        let config = test_config();
        let state = ServerState::new(config, OutputFormat::Json).await.unwrap();
        assert_eq!(state.output_format, OutputFormat::Json);
        assert_eq!(state.config.site, "datadoghq.com");
    }

    #[tokio::test]
    async fn test_server_state_tool_context() {
        let config = test_config();
        let state = ServerState::new(config, OutputFormat::Toon).await.unwrap();
        let ctx = state.tool_context();
        assert_eq!(ctx.output_format, OutputFormat::Toon);
    }

    #[test]
    fn test_tool_context_creation() {
        let config = test_config();
        let client = DatadogClient::new(config).unwrap();
        let ctx = ToolContext::new(Arc::new(client), OutputFormat::Json);
        assert_eq!(ctx.output_format, OutputFormat::Json);
    }

    #[test]
    fn test_tool_context_api_accessors() {
        let config = test_config();
        let client = DatadogClient::new(config).unwrap();
        let ctx = ToolContext::new(Arc::new(client), OutputFormat::Json);

        // Just verify all API accessors work without panicking
        let _ = ctx.metrics_api();
        let _ = ctx.monitors_api();
        let _ = ctx.dashboards_api();
        let _ = ctx.logs_api();
        let _ = ctx.events_api();
        let _ = ctx.infrastructure_api();
        let _ = ctx.downtimes_api();
        let _ = ctx.synthetics_api();
        let _ = ctx.security_api();
        let _ = ctx.incidents_api();
        let _ = ctx.slos_api();
        let _ = ctx.notebooks_api();
        let _ = ctx.teams_api();
        let _ = ctx.users_api();
    }

    #[test]
    fn test_tool_context_clone() {
        let config = test_config();
        let client = DatadogClient::new(config).unwrap();
        let ctx = ToolContext::new(Arc::new(client), OutputFormat::Toon);
        let cloned = ctx.clone();
        assert_eq!(cloned.output_format, ctx.output_format);
    }
}
