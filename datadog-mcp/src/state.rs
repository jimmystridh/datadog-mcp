//! Server state and tool context management
//!
//! Provides shared state for the MCP server including the Datadog client,
//! configuration, and output format. Also provides `ToolContext` which gives
//! tool functions convenient access to API clients.

use anyhow::Result;
use datadog_api::{
    apis::{
        DashboardsApi, DatabaseMonitoringApi, DowntimesApi, EventsApi, IncidentsApi,
        InfrastructureApi, LogsApi, MetricsApi, MonitorsApi, NotebooksApi, SLOsApi, SecurityApi,
        SlackApi, SyntheticsApi, TeamsApi, UsersApi,
    },
    DatadogClient, DatadogConfig,
};
use std::sync::Arc;
use tokio::sync::OnceCell;
use tracing::info;

use crate::browser_session::BrowserSessionService;
use crate::cache::CacheStore;
use crate::output::OutputFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    ReadOnly,
    ReadWrite,
}

impl AccessMode {
    pub const fn allows_writes(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

pub struct ServerState {
    context: OnceCell<ToolContext>,
    output_format: OutputFormat,
    cache: CacheStore,
    access_mode: AccessMode,
}

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub access_mode: AccessMode,
    pub cache: CacheStore,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            access_mode: AccessMode::ReadOnly,
            cache: CacheStore::disabled(),
        }
    }
}

/// Context passed to tool functions containing client and output format
#[derive(Clone)]
pub struct ToolContext {
    pub client: Arc<DatadogClient>,
    pub browser_sessions: BrowserSessionService,
    pub output_format: OutputFormat,
    pub cache: CacheStore,
}

impl ToolContext {
    pub fn new(client: Arc<DatadogClient>, output_format: OutputFormat) -> Self {
        Self {
            client,
            browser_sessions: BrowserSessionService::default(),
            output_format,
            cache: CacheStore::disabled(),
        }
    }

    pub fn with_cache(
        client: Arc<DatadogClient>,
        output_format: OutputFormat,
        cache: CacheStore,
    ) -> Self {
        Self {
            client,
            browser_sessions: BrowserSessionService::default(),
            output_format,
            cache,
        }
    }

    pub fn metrics_api(&self) -> MetricsApi {
        MetricsApi::new((*self.client).clone())
    }

    pub fn database_monitoring_api(&self) -> DatabaseMonitoringApi {
        DatabaseMonitoringApi::new((*self.client).clone())
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

    pub fn slack_api(&self) -> SlackApi {
        SlackApi::new((*self.client).clone())
    }
}

impl ServerState {
    pub fn new(config: DatadogConfig, output_format: OutputFormat) -> Result<Self> {
        Self::new_with_options(config, output_format, ServerOptions::default())
    }

    pub fn new_with_options(
        config: DatadogConfig,
        output_format: OutputFormat,
        options: ServerOptions,
    ) -> Result<Self> {
        let client = DatadogClient::new(config)?;
        let context = OnceCell::new();
        assert!(
            context
                .set(ToolContext::with_cache(
                    Arc::new(client),
                    output_format,
                    options.cache.clone(),
                ))
                .is_ok(),
            "a new context cell must be empty"
        );
        Ok(Self {
            context,
            output_format,
            cache: options.cache,
            access_mode: options.access_mode,
        })
    }

    /// Construct server state without accessing credentials.
    ///
    /// Credential loading is deferred until the first tool call so MCP initialization can
    /// complete without waiting for an OS keyring prompt or Datadog network access.
    pub fn deferred(output_format: OutputFormat, options: ServerOptions) -> Self {
        Self {
            context: OnceCell::new(),
            output_format,
            cache: options.cache,
            access_mode: options.access_mode,
        }
    }

    pub async fn initialize_tool_context(&self) -> Result<ToolContext> {
        let context = self
            .context
            .get_or_try_init(|| async {
                let config = tokio::task::spawn_blocking(DatadogConfig::from_env_or_file)
                    .await
                    .map_err(|error| anyhow::anyhow!("credential loader task failed: {error}"))??;
                info!(site = %config.site, "Loaded Datadog configuration");
                let client = DatadogClient::new(config)?;
                Ok::<_, anyhow::Error>(ToolContext::with_cache(
                    Arc::new(client),
                    self.output_format,
                    self.cache.clone(),
                ))
            })
            .await?;
        Ok(context.clone())
    }

    pub fn tool_context(&self) -> ToolContext {
        self.context
            .get()
            .expect("tool context must be initialized by the server tool wrapper")
            .clone()
    }

    pub const fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    pub const fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    pub fn is_context_initialized(&self) -> bool {
        self.context.initialized()
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.initialize_tool_context()
            .await?
            .client
            .validate_keys()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> DatadogConfig {
        DatadogConfig::new("test_api_key".into(), "test_app_key".into())
    }

    #[test]
    fn test_server_state_creation() {
        let config = test_config();
        let state = ServerState::new(config, OutputFormat::Json).unwrap();
        assert_eq!(state.tool_context().output_format, OutputFormat::Json);
        assert_eq!(state.tool_context().client.config().site, "datadoghq.com");
        assert_eq!(state.access_mode(), AccessMode::ReadOnly);
    }

    #[test]
    fn deferred_state_does_not_load_credentials_during_construction() {
        let state = ServerState::deferred(OutputFormat::Json, ServerOptions::default());

        assert!(state.context.get().is_none());
        assert_eq!(state.output_format(), OutputFormat::Json);
        assert_eq!(state.access_mode(), AccessMode::ReadOnly);
    }

    #[test]
    fn test_server_state_tool_context() {
        let config = test_config();
        let state = ServerState::new(config, OutputFormat::Json).unwrap();
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
