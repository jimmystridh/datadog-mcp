//! MCP Server implementation for Datadog
//!
//! This module defines the MCP server that exposes Datadog tools.
//! Tools are organized by domain but must remain in a single impl block
//! due to rmcp's `#[tool_router]` macro requirements.

use crate::output::{Formattable, OutputFormat};
use crate::state::ServerState;
use crate::tool_inputs::*;
use crate::tools;
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ErrorData, ToolAnnotations},
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::Serialize;
use std::sync::Arc;

/// Format response using the specified format with fallback to JSON
fn format_response<T: Serialize + Formattable>(data: &T, format: OutputFormat) -> String {
    data.format(format)
        .unwrap_or_else(|_| serde_json::to_string_pretty(data).unwrap_or_default())
}

/// Helper macro to reduce boilerplate in tool implementations.
/// Handles the common pattern of: call tool function -> format response -> return success
macro_rules! tool_call {
    ($self:ident, $func:expr) => {{
        let result = match $func.await {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(tool_error = %error, "Datadog MCP tool failed");
                serde_json::json!({
                    "status": "error",
                    "error": error.to_string(),
                })
            }
        };
        let text = format_response(&result, $self.state.output_format);
        let is_error = result.get("status").and_then(|value| value.as_str()) == Some("error");
        let mut response = if is_error {
            CallToolResult::structured_error(result)
        } else {
            CallToolResult::structured(result)
        };
        response.content = vec![ContentBlock::text(text)];
        Ok(response)
    }};
}

macro_rules! write_tool_call {
    ($self:ident, $name:literal, $func:expr) => {{
        if !$self.state.allow_write {
            let result = serde_json::json!({
                "status": "error",
                "error": format!(
                    "{} is disabled because the server is running in read-only mode; restart with --allow-write to enable Datadog mutations",
                    $name
                ),
            });
            Ok(CallToolResult::structured_error(result))
        } else {
            tool_call!($self, $func)
        }
    }};
}

#[derive(Clone)]
pub struct DatadogMcpServer {
    pub state: Arc<ServerState>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DatadogMcpServer {
    pub fn new(state: ServerState) -> Self {
        let mut tool_router = Self::tool_router();
        for route in tool_router.map.values_mut() {
            route.attr.annotations = Some(
                ToolAnnotations::new()
                    .read_only(true)
                    .destructive(false)
                    .idempotent(true)
                    .open_world(true),
            );
        }

        let write_tools = [
            ("create_monitor", false, false),
            ("update_monitor", true, true),
            ("delete_monitor", true, true),
            ("create_dashboard", false, false),
            ("update_dashboard", true, true),
            ("delete_dashboard", true, true),
            ("create_event", false, false),
            ("create_downtime", false, false),
            ("cancel_downtime", true, true),
            ("create_synthetics_test", false, false),
            ("update_synthetics_test", true, true),
            ("trigger_synthetics_tests", false, false),
            ("delete_synthetics_tests", true, true),
            ("cleanup_cache", true, true),
        ];
        for (name, destructive, idempotent) in write_tools {
            if let Some(route) = tool_router.map.get_mut(name) {
                route.attr.annotations = Some(
                    ToolAnnotations::new()
                        .read_only(false)
                        .destructive(destructive)
                        .idempotent(idempotent)
                        .open_world(name != "cleanup_cache"),
                );
            }
        }
        for name in ["analyze_data", "cleanup_cache"] {
            if let Some(route) = tool_router.map.get_mut(name) {
                if let Some(annotations) = route.attr.annotations.as_mut() {
                    annotations.open_world_hint = Some(false);
                }
            }
        }

        Self {
            state: Arc::new(state),
            tool_router,
        }
    }

    // ============================================================================
    // VALIDATION
    // ============================================================================

    #[tool(description = "Validate Datadog API credentials")]
    pub async fn validate_api_key(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::validate_api_key(self.state.tool_context()))
    }

    // ============================================================================
    // METRICS
    // ============================================================================

    #[tool(description = "Query Datadog metrics time series data")]
    pub async fn get_metrics(
        &self,
        Parameters(input): Parameters<GetMetricsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_metrics(
                self.state.tool_context(),
                input.query,
                input.from_timestamp,
                input.to_timestamp,
            )
        )
    }

    #[tool(description = "Search for metrics by name pattern")]
    pub async fn search_metrics(
        &self,
        Parameters(input): Parameters<SearchMetricsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::search_metrics(self.state.tool_context(), input.query, input.from_timestamp,)
        )
    }

    #[tool(description = "Get metadata for a specific metric")]
    pub async fn get_metric_metadata(
        &self,
        Parameters(input): Parameters<GetMetricMetadataInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_metric_metadata(self.state.tool_context(), input.metric_name)
        )
    }

    // ============================================================================
    // MONITORS
    // ============================================================================

    #[tool(description = "Get all Datadog monitors")]
    pub async fn get_monitors(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_monitors(self.state.tool_context()))
    }

    #[tool(description = "Search Datadog monitors")]
    pub async fn search_monitors(
        &self,
        Parameters(input): Parameters<SearchMonitorsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::search_monitors(
                self.state.tool_context(),
                input.query,
                input.page,
                input.per_page,
                input.sort,
            )
        )
    }

    #[tool(description = "Get specific monitor by ID")]
    pub async fn get_monitor(
        &self,
        Parameters(input): Parameters<GetMonitorInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_monitor(self.state.tool_context(), input.monitor_id)
        )
    }

    #[tool(description = "Create a new Datadog monitor")]
    pub async fn create_monitor(
        &self,
        Parameters(input): Parameters<CreateMonitorInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "create_monitor",
            tools::create_monitor(
                self.state.tool_context(),
                input.name,
                input.monitor_type,
                input.query,
                input.message,
                input.tags,
                input.options,
            )
        )
    }

    #[tool(description = "Update an existing Datadog monitor")]
    pub async fn update_monitor(
        &self,
        Parameters(input): Parameters<UpdateMonitorInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "update_monitor",
            tools::update_monitor(
                self.state.tool_context(),
                input.monitor_id,
                input.name,
                input.query,
                input.message,
                input.tags,
                input.options,
            )
        )
    }

    #[tool(description = "Delete a monitor")]
    pub async fn delete_monitor(
        &self,
        Parameters(input): Parameters<DeleteMonitorInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "delete_monitor",
            tools::delete_monitor(self.state.tool_context(), input.monitor_id)
        )
    }

    // ============================================================================
    // DASHBOARDS
    // ============================================================================

    #[tool(description = "Get all Datadog dashboards")]
    pub async fn get_dashboards(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_dashboards(self.state.tool_context()))
    }

    #[tool(description = "Get specific dashboard by ID")]
    pub async fn get_dashboard(
        &self,
        Parameters(input): Parameters<GetDashboardInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_dashboard(self.state.tool_context(), input.dashboard_id)
        )
    }

    #[tool(description = "Create a new dashboard")]
    pub async fn create_dashboard(
        &self,
        Parameters(input): Parameters<CreateDashboardInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "create_dashboard",
            tools::create_dashboard(
                self.state.tool_context(),
                input.title,
                input.layout_type,
                input.widgets,
                input.description,
            )
        )
    }

    #[tool(description = "Update an existing dashboard")]
    pub async fn update_dashboard(
        &self,
        Parameters(input): Parameters<UpdateDashboardInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "update_dashboard",
            tools::update_dashboard(
                self.state.tool_context(),
                input.dashboard_id,
                input.title,
                input.widgets,
            )
        )
    }

    #[tool(description = "Delete a dashboard")]
    pub async fn delete_dashboard(
        &self,
        Parameters(input): Parameters<DeleteDashboardInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "delete_dashboard",
            tools::delete_dashboard(self.state.tool_context(), input.dashboard_id)
        )
    }

    // ============================================================================
    // LOGS & EVENTS
    // ============================================================================

    #[tool(description = "Search Datadog logs")]
    pub async fn search_logs(
        &self,
        Parameters(input): Parameters<SearchLogsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::search_logs(
                self.state.tool_context(),
                input.query,
                input.from_time,
                input.to_time,
                input.limit,
                input.cursor,
            )
        )
    }

    #[tool(description = "Get Datadog events")]
    pub async fn get_events(
        &self,
        Parameters(input): Parameters<GetEventsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_events(
                self.state.tool_context(),
                input.start,
                input.end,
                input.priority,
                input.sources,
            )
        )
    }

    #[tool(description = "Create a Datadog event")]
    pub async fn create_event(
        &self,
        Parameters(input): Parameters<CreateEventInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "create_event",
            tools::create_event(
                self.state.tool_context(),
                input.title,
                input.text,
                input.tags,
                input.alert_type,
                input.priority,
                input.host,
                input.source_type_name,
                input.aggregation_key,
                input.date_happened,
                input.device_name,
                input.related_event_id,
            )
        )
    }

    #[tool(description = "Get Datadog event by ID")]
    pub async fn get_event(
        &self,
        Parameters(input): Parameters<GetEventInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_event(self.state.tool_context(), input.event_id)
        )
    }

    // ============================================================================
    // INFRASTRUCTURE
    // ============================================================================

    #[tool(description = "Get infrastructure and hosts information")]
    pub async fn get_infrastructure(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_infrastructure(self.state.tool_context()))
    }

    #[tool(description = "Get host tags")]
    pub async fn get_tags(
        &self,
        Parameters(input): Parameters<GetTagsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_tags(self.state.tool_context(), input.source)
        )
    }

    #[tool(description = "Get Kubernetes deployments with their current state")]
    pub async fn get_kubernetes_deployments(
        &self,
        Parameters(input): Parameters<GetKubernetesDeploymentsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_kubernetes_deployments(self.state.tool_context(), input.namespace)
        )
    }

    // ============================================================================
    // DOWNTIMES
    // ============================================================================

    #[tool(description = "Get scheduled downtimes")]
    pub async fn get_downtimes(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_downtimes(self.state.tool_context()))
    }

    #[tool(description = "Create a scheduled downtime")]
    pub async fn create_downtime(
        &self,
        Parameters(input): Parameters<CreateDowntimeInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "create_downtime",
            tools::create_downtime(
                self.state.tool_context(),
                input.scope,
                input.start,
                input.end,
                input.message,
            )
        )
    }

    #[tool(description = "Cancel a scheduled downtime")]
    pub async fn cancel_downtime(
        &self,
        Parameters(input): Parameters<CancelDowntimeInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "cancel_downtime",
            tools::cancel_downtime(self.state.tool_context(), input.downtime_id)
        )
    }

    // ============================================================================
    // SYNTHETICS
    // ============================================================================

    #[tool(description = "Get all Synthetics tests")]
    pub async fn get_synthetics_tests(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_synthetics_tests(self.state.tool_context()))
    }

    #[tool(description = "Get all available Synthetics testing locations")]
    pub async fn get_synthetics_locations(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_synthetics_locations(self.state.tool_context())
        )
    }

    #[tool(description = "Create a Synthetic API test (HTTP check)")]
    pub async fn create_synthetics_test(
        &self,
        Parameters(input): Parameters<CreateSyntheticsTestInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "create_synthetics_test",
            tools::create_synthetics_test(
                self.state.tool_context(),
                input.name,
                input.test_type,
                input.url,
                input.locations,
                input.message,
                input.tags,
                input.tick_every,
            )
        )
    }

    #[tool(description = "Update an existing Synthetics test")]
    pub async fn update_synthetics_test(
        &self,
        Parameters(input): Parameters<UpdateSyntheticsTestInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "update_synthetics_test",
            tools::update_synthetics_test(
                self.state.tool_context(),
                input.public_id,
                input.name,
                input.url,
                input.locations,
                input.message,
                input.tags,
                input.tick_every,
            )
        )
    }

    #[tool(description = "Trigger Synthetics tests on-demand")]
    pub async fn trigger_synthetics_tests(
        &self,
        Parameters(input): Parameters<TriggerSyntheticsTestsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "trigger_synthetics_tests",
            tools::trigger_synthetics_tests(self.state.tool_context(), input.test_ids)
        )
    }

    #[tool(description = "Delete Synthetics tests")]
    pub async fn delete_synthetics_tests(
        &self,
        Parameters(input): Parameters<DeleteSyntheticsTestsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "delete_synthetics_tests",
            tools::delete_synthetics_tests(
                self.state.tool_context(),
                input.test_ids,
                input.force_delete_dependencies,
            )
        )
    }

    // ============================================================================
    // SECURITY & INCIDENTS
    // ============================================================================

    #[tool(description = "Get security monitoring rules")]
    pub async fn get_security_rules(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_security_rules(self.state.tool_context()))
    }

    #[tool(description = "Get incidents with pagination support")]
    pub async fn get_incidents(
        &self,
        Parameters(input): Parameters<GetIncidentsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_incidents(
                self.state.tool_context(),
                input.page_size,
                input.page_offset,
            )
        )
    }

    // ============================================================================
    // SLOS & NOTEBOOKS
    // ============================================================================

    #[tool(description = "Get Service Level Objectives")]
    pub async fn get_slos(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_slos(self.state.tool_context()))
    }

    #[tool(description = "Get Datadog notebooks")]
    pub async fn get_notebooks(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_notebooks(self.state.tool_context()))
    }

    // ============================================================================
    // TEAMS & USERS
    // ============================================================================

    #[tool(description = "Get teams")]
    pub async fn get_teams(
        &self,
        Parameters(input): Parameters<GetTeamsInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_teams(
                self.state.tool_context(),
                input.page_number,
                input.page_size,
            )
        )
    }

    #[tool(description = "Get users")]
    pub async fn get_users(
        &self,
        Parameters(input): Parameters<GetUsersInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_users(
                self.state.tool_context(),
                input.page_number,
                input.page_size,
            )
        )
    }

    // ============================================================================
    // UTILITIES
    // ============================================================================

    #[tool(description = "Analyze stored Datadog data (summary, stats, or trends)")]
    pub async fn analyze_data(
        &self,
        Parameters(input): Parameters<AnalyzeDataInput>,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::analyze_data(
                self.state.tool_context(),
                input.filepath,
                input.analysis_type
            )
        )
    }

    #[tool(description = "Clean up old cache files")]
    pub async fn cleanup_cache(
        &self,
        Parameters(input): Parameters<CleanupCacheInput>,
    ) -> Result<CallToolResult, ErrorData> {
        write_tool_call!(
            self,
            "cleanup_cache",
            tools::cleanup_cache_tool(self.state.tool_context(), input.older_than_hours)
        )
    }
}

// ============================================================================
// SERVER HANDLER
// ============================================================================

#[tool_handler(
    router = self.tool_router,
    name = "datadog-mcp",
    instructions = "This server provides comprehensive access to Datadog's monitoring and observability platform. Use the available tools to query metrics, manage monitors and dashboards, search logs, retrieve infrastructure information, manage incidents, and more."
)]
impl ServerHandler for DatadogMcpServer {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::OutputFormat;
    use datadog_api::DatadogConfig;
    use rmcp::model::ProtocolVersion;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn server_info_uses_current_package_and_protocol() {
        let config = DatadogConfig::new("api-key".into(), "app-key".into());
        let state = ServerState::new(config, OutputFormat::Json).await.unwrap();
        let server = DatadogMcpServer::new(state);

        let info = server.get_info();

        assert_eq!(info.server_info.name, "datadog-mcp");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
        assert!(info.capabilities.tools.is_some());
        assert!(info.instructions.is_some());
        assert!(server.tool_router.list_all().len() >= 35);
    }

    #[tokio::test]
    async fn write_tools_are_annotated_and_blocked_by_default() {
        let config = DatadogConfig::new("api-key".into(), "app-key".into());
        let state = ServerState::new(config, OutputFormat::Json).await.unwrap();
        let server = DatadogMcpServer::new(state);

        let tool = server.tool_router.get("create_monitor").unwrap();
        let annotations = tool.annotations.as_ref().unwrap();
        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(false));

        let result = server
            .create_monitor(Parameters(CreateMonitorInput {
                name: "Blocked".to_string(),
                monitor_type: "metric alert".to_string(),
                query: "avg(last_5m):avg:system.cpu.user{*} > 90".to_string(),
                message: None,
                tags: None,
                options: None,
            }))
            .await
            .unwrap();
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.structured_content.unwrap()["status"], "error");
    }

    #[tokio::test]
    async fn datadog_api_failures_are_tool_errors() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/monitor/42"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&mock_server)
            .await;

        let config =
            DatadogConfig::new("api-key".into(), "app-key".into()).with_base_url(mock_server.uri());
        let state = ServerState::new(config, OutputFormat::Json).await.unwrap();
        let server = DatadogMcpServer::new(state);
        let result = server
            .get_monitor(Parameters(GetMonitorInput {
                monitor_id: crate::ids::MonitorId(42),
            }))
            .await
            .unwrap();

        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.structured_content.unwrap()["status"], "error");
    }
}
