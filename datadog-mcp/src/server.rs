//! MCP Server implementation for Datadog
//!
//! This module defines the MCP server that exposes Datadog tools.
//! Tools are organized by domain but must remain in a single impl block
//! due to rmcp's `#[tool_box]` macro requirements.

use crate::errors::to_mcp_error;
use crate::output::{Formattable, OutputFormat};
use crate::state::ServerState;
use crate::tool_inputs::*;
use crate::tools;
use rmcp::{
    model::{
        CallToolResult, Content, ErrorData, Implementation, InitializeRequestParam,
        InitializeResult, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, Error as McpError, RoleServer, ServerHandler,
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
        let result = $func.await.map_err(to_mcp_error)?;
        let text = format_response(&result, $self.state.output_format);
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }};
}

#[derive(Clone)]
pub struct DatadogMcpServer {
    pub state: Arc<ServerState>,
}

#[tool(tool_box)]
impl DatadogMcpServer {
    pub fn new(state: ServerState) -> Self {
        Self {
            state: Arc::new(state),
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
        #[tool(aggr)] input: GetMetricsInput,
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
        #[tool(aggr)] input: SearchMetricsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::search_metrics(self.state.tool_context(), input.query)
        )
    }

    #[tool(description = "Get metadata for a specific metric")]
    pub async fn get_metric_metadata(
        &self,
        #[tool(aggr)] input: GetMetricMetadataInput,
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
        #[tool(aggr)] input: SearchMonitorsInput,
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
        #[tool(aggr)] input: GetMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_monitor(self.state.tool_context(), input.monitor_id)
        )
    }

    #[tool(description = "Create a new Datadog monitor")]
    pub async fn create_monitor(
        &self,
        #[tool(aggr)] input: CreateMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::create_monitor(
                self.state.tool_context(),
                input.name,
                input.monitor_type,
                input.query,
                input.message,
                input.options,
            )
        )
    }

    #[tool(description = "Update an existing Datadog monitor")]
    pub async fn update_monitor(
        &self,
        #[tool(aggr)] input: UpdateMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::update_monitor(
                self.state.tool_context(),
                input.monitor_id,
                input.name,
                input.query,
                input.message,
                input.options,
            )
        )
    }

    #[tool(description = "Delete a monitor")]
    pub async fn delete_monitor(
        &self,
        #[tool(aggr)] input: DeleteMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: GetDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_dashboard(self.state.tool_context(), input.dashboard_id)
        )
    }

    #[tool(description = "Create a new dashboard")]
    pub async fn create_dashboard(
        &self,
        #[tool(aggr)] input: CreateDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: UpdateDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: DeleteDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::delete_dashboard(self.state.tool_context(), input.dashboard_id)
        )
    }

    // ============================================================================
    // LOGS & EVENTS
    // ============================================================================

    #[tool(description = "Search Datadog logs")]
    pub async fn search_logs(
        &self,
        #[tool(aggr)] input: SearchLogsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::search_logs(
                self.state.tool_context(),
                input.query,
                input.from_time,
                input.to_time,
                input.limit,
            )
        )
    }

    #[tool(description = "Get Datadog events")]
    pub async fn get_events(
        &self,
        #[tool(aggr)] input: GetEventsInput,
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
        #[tool(aggr)] input: CreateEventInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: GetEventInput,
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
        #[tool(aggr)] input: GetTagsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_tags(self.state.tool_context(), input.source)
        )
    }

    #[tool(description = "Get Kubernetes deployments with their current state")]
    pub async fn get_kubernetes_deployments(
        &self,
        #[tool(aggr)] input: GetKubernetesDeploymentsInput,
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
        #[tool(aggr)] input: CreateDowntimeInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: CancelDowntimeInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: CreateSyntheticsTestInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: UpdateSyntheticsTestInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: TriggerSyntheticsTestsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::trigger_synthetics_tests(self.state.tool_context(), input.test_ids)
        )
    }

    #[tool(description = "Delete Synthetics tests")]
    pub async fn delete_synthetics_tests(
        &self,
        #[tool(aggr)] input: DeleteSyntheticsTestsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
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
        #[tool(aggr)] input: GetIncidentsInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::get_incidents(self.state.tool_context(), input.page_size)
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
    pub async fn get_teams(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_teams(self.state.tool_context()))
    }

    #[tool(description = "Get users")]
    pub async fn get_users(&self) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::get_users(self.state.tool_context()))
    }

    // ============================================================================
    // UTILITIES
    // ============================================================================

    #[tool(description = "Analyze stored Datadog data (summary, stats, or trends)")]
    pub async fn analyze_data(
        &self,
        #[tool(aggr)] input: AnalyzeDataInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(
            self,
            tools::analyze_data(input.filepath, input.analysis_type)
        )
    }

    #[tool(description = "Clean up old cache files")]
    pub async fn cleanup_cache(
        &self,
        #[tool(aggr)] input: CleanupCacheInput,
    ) -> Result<CallToolResult, ErrorData> {
        tool_call!(self, tools::cleanup_cache_tool(input.older_than_hours))
    }
}

// ============================================================================
// SERVER HANDLER
// ============================================================================

#[tool(tool_box)]
impl ServerHandler for DatadogMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides comprehensive access to Datadog's monitoring and \
                 observability platform. Use the available tools to query metrics, manage \
                 monitors and dashboards, search logs, retrieve infrastructure information, \
                 manage incidents, and more."
                    .to_string(),
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
