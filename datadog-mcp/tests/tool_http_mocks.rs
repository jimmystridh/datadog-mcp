use datadog_api::{config::DatadogConfig, DatadogClient};
use datadog_mcp::state::ToolContext;
use datadog_mcp::tool_inputs::MonitorId;
use datadog_mcp::{tools, tools_part2};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

#[tokio::test]
async fn get_metrics_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "series": [
                {
                    "pointlist": [[1, 2.0], [2, 3.0]],
                    "scope": "host:local"
                }
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_metrics(ctx, "test".into(), 0, 10).await.unwrap();
    assert_eq!(out["status"], "success");
    assert!(out["filepath"].as_str().unwrap().contains("metrics_"));
    assert_eq!(out["series_count"], 1);
    assert_eq!(out["data_points"], 2);
}

#[tokio::test]
async fn get_metrics_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/query"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_metrics(ctx, "test".into(), 0, 10).await.unwrap();
    assert_eq!(out["status"], "error");
}

#[tokio::test]
async fn get_monitor_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/42"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_monitor(ctx, MonitorId(42)).await.unwrap();
    assert_eq!(out["status"], "error");
}

#[tokio::test]
async fn search_logs_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/logs/events/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                { "id": "1" },
                { "id": "2" }
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools_part2::search_logs(
        ctx,
        "env:prod".into(),
        "now-1h".into(),
        "now".into(),
        Some(10),
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["log_count"], 2);
}
