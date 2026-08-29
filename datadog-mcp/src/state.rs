//! Server state and tool context management
//!
//! Provides shared state for the MCP server including the Datadog client,
//! configuration, and output format. Also provides `ToolContext` which gives
//! tool functions convenient access to API clients.

use anyhow::Result;
use datadog_api::{
    apis::{
        DashboardsApi, DowntimesApi, EventsApi, IncidentsApi, InfrastructureApi, LogsApi,
        MetricsApi, MonitorsApi, NotebooksApi, SLOsApi, SecurityApi, SyntheticsApi, TeamsApi,
        UsersApi,
    },
    DatadogClient, DatadogConfig,
};
use std::path::PathBuf;
use std::sync::Arc;

use crate::output::OutputFormat;

pub struct ServerState {
    pub client: Arc<DatadogClient>,
    pub config: DatadogConfig,
    pub output_format: OutputFormat,
    pub allow_write: bool,
    pub cache_responses: bool,
    pub cache_dir: PathBuf,
    pub cache_max_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub allow_write: bool,
    pub cache_responses: bool,
    pub cache_dir: PathBuf,
    pub cache_max_bytes: u64,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            allow_write: false,
            cache_responses: false,
            cache_dir: crate::cache::default_cache_dir(),
            cache_max_bytes: 100 * 1024 * 1024,
        }
    }
}

/// Context passed to tool functions containing client and output format
#[derive(Clone)]
pub struct ToolContext {
    pub client: Arc<DatadogClient>,
    pub output_format: OutputFormat,
    pub allow_write: bool,
    pub cache_responses: bool,
    pub cache_dir: PathBuf,
    pub cache_max_bytes: u64,
}

impl ToolContext {
    pub fn new(client: Arc<DatadogClient>, output_format: OutputFormat) -> Self {
        Self {
            client,
            output_format,
            allow_write: false,
            cache_responses: false,
            cache_dir: crate::cache::default_cache_dir(),
            cache_max_bytes: 100 * 1024 * 1024,
        }
    }

    pub fn with_options(
        client: Arc<DatadogClient>,
        output_format: OutputFormat,
        options: &ServerOptions,
    ) -> Self {
        Self {
            client,
            output_format,
            allow_write: options.allow_write,
            cache_responses: options.cache_responses,
            cache_dir: options.cache_dir.clone(),
            cache_max_bytes: options.cache_max_bytes,
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
        Self::new_with_options(config, output_format, ServerOptions::default()).await
    }

    pub async fn new_with_options(
        config: DatadogConfig,
        output_format: OutputFormat,
        options: ServerOptions,
    ) -> Result<Self> {
        let client = DatadogClient::new(config.clone())?;
        Ok(Self {
            client: Arc::new(client),
            config,
            output_format,
            allow_write: options.allow_write,
            cache_responses: options.cache_responses,
            cache_dir: options.cache_dir,
            cache_max_bytes: options.cache_max_bytes,
        })
    }

    pub fn tool_context(&self) -> ToolContext {
        ToolContext::with_options(
            self.client.clone(),
            self.output_format,
            &ServerOptions {
                allow_write: self.allow_write,
                cache_responses: self.cache_responses,
                cache_dir: self.cache_dir.clone(),
                cache_max_bytes: self.cache_max_bytes,
            },
        )
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.client.validate_keys().await?;
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
        let state = ServerState::new(config, OutputFormat::Json).await.unwrap();
        let ctx = state.tool_context();
        assert_eq!(ctx.output_format, OutputFormat::Json);
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
        let ctx = ToolContext::new(Arc::new(client), OutputFormat::Json);
        let cloned = ctx.clone();
        assert_eq!(cloned.output_format, ctx.output_format);
    }
}
