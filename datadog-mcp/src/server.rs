use crate::state::ServerState;
use crate::tool_inputs::*;
use crate::tools;
use crate::tools_part2;
use rmcp::{
    model::{
        CallToolResult, Content, ErrorCode, ErrorData, Implementation, InitializeRequestParam,
        InitializeResult, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, Error as McpError, RoleServer, ServerHandler,
};
use std::sync::Arc;

fn to_mcp_error(err: anyhow::Error) -> ErrorData {
    ErrorData::new(ErrorCode(-32603), err.to_string(), None)
}

#[allow(dead_code)]
fn api_error(code: &str, operation: &str, error: impl std::fmt::Display) -> ErrorData {
    ErrorData::new(
        ErrorCode(-32603),
        format!(
            "{}: Failed to {}. {}. Please check your API credentials and permissions.",
            code, operation, error
        ),
        None,
    )
}

#[allow(dead_code)]
fn validation_error(message: &str) -> ErrorData {
    ErrorData::new(
        ErrorCode(-32602),
        format!("VALIDATION_ERROR: {}", message),
        None,
    )
}

#[allow(dead_code)]
fn not_found_error(resource: &str, id: &str) -> ErrorData {
    ErrorData::new(
        ErrorCode(-32602),
        format!("NOT_FOUND: {} with ID '{}' not found", resource, id),
        None,
    )
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
    // METRICS & MONITORING TOOLS
    // ============================================================================

    #[tool(description = "Validate Datadog API credentials")]
    pub async fn validate_api_key(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools::validate_api_key(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Query Datadog metrics time series data")]
    pub async fn get_metrics(
        &self,
        #[tool(aggr)] input: GetMetricsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_metrics(
            self.state.client.clone(),
            input.query,
            input.from_timestamp,
            input.to_timestamp,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Search for metrics by name pattern")]
    pub async fn search_metrics(
        &self,
        #[tool(aggr)] input: SearchMetricsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::search_metrics(self.state.client.clone(), input.query)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get metadata for a specific metric")]
    pub async fn get_metric_metadata(
        &self,
        #[tool(aggr)] input: GetMetricMetadataInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_metric_metadata(self.state.client.clone(), input.metric_name)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get all Datadog monitors")]
    pub async fn get_monitors(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_monitors(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get specific monitor by ID")]
    pub async fn get_monitor(
        &self,
        #[tool(aggr)] input: GetMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_monitor(self.state.client.clone(), input.monitor_id)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Create a new Datadog monitor")]
    pub async fn create_monitor(
        &self,
        #[tool(aggr)] input: CreateMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::create_monitor(
            self.state.client.clone(),
            input.name,
            input.monitor_type,
            input.query,
            input.message,
            input.options,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Update an existing Datadog monitor")]
    pub async fn update_monitor(
        &self,
        #[tool(aggr)] input: UpdateMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::update_monitor(
            self.state.client.clone(),
            input.monitor_id,
            input.name,
            input.query,
            input.message,
            input.options,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Delete a monitor")]
    pub async fn delete_monitor(
        &self,
        #[tool(aggr)] input: DeleteMonitorInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::delete_monitor(self.state.client.clone(), input.monitor_id)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // DASHBOARD TOOLS
    // ============================================================================

    #[tool(description = "Get all Datadog dashboards")]
    pub async fn get_dashboards(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_dashboards(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get specific dashboard by ID")]
    pub async fn get_dashboard(
        &self,
        #[tool(aggr)] input: GetDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::get_dashboard(self.state.client.clone(), input.dashboard_id)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Create a new dashboard")]
    pub async fn create_dashboard(
        &self,
        #[tool(aggr)] input: CreateDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::create_dashboard(
            self.state.client.clone(),
            input.title,
            input.layout_type,
            input.widgets,
            input.description,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Update an existing dashboard")]
    pub async fn update_dashboard(
        &self,
        #[tool(aggr)] input: UpdateDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::update_dashboard(
            self.state.client.clone(),
            input.dashboard_id,
            input.title,
            input.widgets,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Delete a dashboard")]
    pub async fn delete_dashboard(
        &self,
        #[tool(aggr)] input: DeleteDashboardInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools::delete_dashboard(self.state.client.clone(), input.dashboard_id)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // LOGS & EVENTS TOOLS
    // ============================================================================

    #[tool(description = "Search Datadog logs")]
    pub async fn search_logs(
        &self,
        #[tool(aggr)] input: SearchLogsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::search_logs(
            self.state.client.clone(),
            input.query,
            input.from_time,
            input.to_time,
            input.limit,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get Datadog events")]
    pub async fn get_events(
        &self,
        #[tool(aggr)] input: GetEventsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_events(
            self.state.client.clone(),
            input.start,
            input.end,
            input.priority,
            input.sources,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // INFRASTRUCTURE & TAGS TOOLS
    // ============================================================================

    #[tool(description = "Get infrastructure and hosts information")]
    pub async fn get_infrastructure(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_infrastructure(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get host tags")]
    pub async fn get_tags(
        &self,
        #[tool(aggr)] input: GetTagsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_tags(self.state.client.clone(), input.source)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get Kubernetes deployments with their current state")]
    pub async fn get_kubernetes_deployments(
        &self,
        #[tool(aggr)] input: GetKubernetesDeploymentsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result =
            tools_part2::get_kubernetes_deployments(self.state.client.clone(), input.namespace)
                .await
                .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get scheduled downtimes")]
    pub async fn get_downtimes(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_downtimes(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Create a scheduled downtime")]
    pub async fn create_downtime(
        &self,
        #[tool(aggr)] input: CreateDowntimeInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::create_downtime(
            self.state.client.clone(),
            input.scope,
            input.start,
            input.end,
            input.message,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Cancel a scheduled downtime")]
    pub async fn cancel_downtime(
        &self,
        #[tool(aggr)] input: CancelDowntimeInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::cancel_downtime(self.state.client.clone(), input.downtime_id)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // TESTING TOOLS
    // ============================================================================

    #[tool(description = "Get all Synthetics tests")]
    pub async fn get_synthetics_tests(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_synthetics_tests(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get all available Synthetics testing locations")]
    pub async fn get_synthetics_locations(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_synthetics_locations(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Create a Synthetic API test (HTTP check)")]
    pub async fn create_synthetics_test(
        &self,
        #[tool(aggr)] input: CreateSyntheticsTestInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::create_synthetics_test(
            self.state.client.clone(),
            input.name,
            input.test_type,
            input.url,
            input.locations,
            input.message,
            input.tags,
            input.tick_every,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Update an existing Synthetics test")]
    pub async fn update_synthetics_test(
        &self,
        #[tool(aggr)] input: UpdateSyntheticsTestInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::update_synthetics_test(
            self.state.client.clone(),
            input.public_id,
            input.name,
            input.url,
            input.locations,
            input.message,
            input.tags,
            input.tick_every,
        )
        .await
        .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Trigger Synthetics tests on-demand")]
    pub async fn trigger_synthetics_tests(
        &self,
        #[tool(aggr)] input: TriggerSyntheticsTestsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result =
            tools_part2::trigger_synthetics_tests(self.state.client.clone(), input.test_ids)
                .await
                .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // SECURITY & INCIDENTS TOOLS
    // ============================================================================

    #[tool(description = "Get security monitoring rules")]
    pub async fn get_security_rules(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_security_rules(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get incidents with pagination support")]
    pub async fn get_incidents(
        &self,
        #[tool(aggr)] input: GetIncidentsInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_incidents(self.state.client.clone(), input.page_size)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get Service Level Objectives")]
    pub async fn get_slos(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_slos(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get Datadog notebooks")]
    pub async fn get_notebooks(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_notebooks(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // TEAMS & USERS TOOLS
    // ============================================================================

    #[tool(description = "Get teams")]
    pub async fn get_teams(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_teams(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Get users")]
    pub async fn get_users(&self) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::get_users(self.state.client.clone())
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    // ============================================================================
    // UTILITIES TOOLS
    // ============================================================================

    #[tool(description = "Analyze stored Datadog data (summary, stats, or trends)")]
    pub async fn analyze_data(
        &self,
        #[tool(aggr)] input: AnalyzeDataInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::analyze_data(input.filepath, input.analysis_type)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Clean up old cache files")]
    pub async fn cleanup_cache(
        &self,
        #[tool(aggr)] input: CleanupCacheInput,
    ) -> Result<CallToolResult, ErrorData> {
        let result = tools_part2::cleanup_cache_tool(input.older_than_hours)
            .await
            .map_err(to_mcp_error)?;

        let text = serde_json::to_string_pretty(&result).unwrap_or_default();
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

// Implement ServerHandler trait to provide server metadata and capabilities
#[tool(tool_box)]
impl ServerHandler for DatadogMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "This server provides comprehensive access to Datadog's monitoring and observability platform. \
                Use the available tools to query metrics, manage monitors and dashboards, search logs, \
                retrieve infrastructure information, manage incidents, and more.".to_string()
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

    // The tool_box macro will automatically implement list_tools and call_tool
}
