use datadog_api::{config::DatadogConfig, DatadogClient};
use datadog_mcp::response::render_tool_result;
use datadog_mcp::state::ToolContext;
use datadog_mcp::tool_inputs::{DashboardId, DowntimeId, MonitorId, SyntheticsTestId};
use datadog_mcp::tools;
use serde_json::json;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

macro_rules! call_tool {
    ($context:ident, $call:expr) => {{
        let result = $call.await;
        render_tool_result(result, &$context)
            .await
            .structured_content
            .expect("tool responses are structured")
    }};
}

fn assert_success(out: &serde_json::Value) {
    assert_eq!(out["status"], "success");
}

fn mock_config(base_url: &str) -> DatadogConfig {
    DatadogConfig::new("test_api_key".into(), "test_app_key".into())
        .with_base_url(base_url.to_string())
}

async fn mock_context(server: &MockServer) -> ToolContext {
    let cfg = mock_config(&server.uri());
    let client = DatadogClient::new(cfg).expect("client");
    ToolContext::new(
        std::sync::Arc::new(client),
        datadog_mcp::output::OutputFormat::Json,
    )
}

#[path = "tool_http/platform.rs"]
mod platform;
#[path = "tool_http/queries.rs"]
mod queries;
#[path = "tool_http/resources.rs"]
mod resources;
#[path = "tool_http/validation.rs"]
mod validation;
