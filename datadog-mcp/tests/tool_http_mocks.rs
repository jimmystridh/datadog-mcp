use datadog_api::{config::DatadogConfig, DatadogClient};
use datadog_mcp::state::ToolContext;
use datadog_mcp::tool_inputs::{DashboardId, DowntimeId, MonitorId, SyntheticsTestId};
use datadog_mcp::tools;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn assert_success(out: &serde_json::Value) {
    let status = out["status"].as_str().unwrap_or("");
    let success_statuses = ["success", "created", "deleted", "cancelled", "ok", "live", "updated"];
    assert!(
        success_statuses.contains(&status),
        "Expected success-like status, got: {}",
        status
    );
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
    let out = tools::search_logs(
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

// ============================================================================
// Monitor Tests
// ============================================================================

#[tokio::test]
async fn get_monitors_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id": 1,
                "name": "CPU Alert",
                "overall_state": "OK",
                "monitor_type": "metric alert"
            },
            {
                "id": 2,
                "name": "Memory Alert",
                "overall_state": "Alert",
                "monitor_type": "metric alert"
            }
        ])))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_monitors(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_monitors"], 2);
    assert_eq!(out["alerting_count"], 1);
}

#[tokio::test]
async fn get_monitor_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 123,
            "name": "Test Monitor",
            "overall_state": "OK",
            "monitor_type": "metric alert",
            "query": "avg(last_5m):avg:system.cpu.user{*} > 80"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_monitor(ctx, MonitorId(123)).await.unwrap();
    assert_success(&out);
    assert_eq!(out["monitor_id"], 123);
    assert_eq!(out["monitor_name"], "Test Monitor");
}

#[tokio::test]
async fn create_monitor_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 456,
            "name": "New Monitor",
            "overall_state": "No Data",
            "monitor_type": "metric alert"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::create_monitor(
        ctx,
        "New Monitor".into(),
        "metric alert".into(),
        "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
        Some("Alert message".into()),
        None,
    )
    .await
    .unwrap();
    assert_success(&out);
    assert_eq!(out["monitor_id"], 456);
}

#[tokio::test]
async fn delete_monitor_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/monitor/789"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted_monitor_id": 789})))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::delete_monitor(ctx, MonitorId(789)).await.unwrap();
    assert_success(&out);
}

// ============================================================================
// Dashboard Tests
// ============================================================================

#[tokio::test]
async fn get_dashboards_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "dashboards": [
                {"id": "abc-123", "title": "Main Dashboard"},
                {"id": "def-456", "title": "API Metrics"}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_dashboards(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_dashboards"], 2);
}

#[tokio::test]
async fn get_dashboard_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard/abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "abc-123",
            "title": "Main Dashboard",
            "layout_type": "ordered",
            "widgets": [
                {"id": 1, "definition": {"type": "note", "content": "Hello"}},
                {"id": 2, "definition": {"type": "note", "content": "World"}}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_dashboard(ctx, DashboardId("abc-123".into()))
        .await
        .unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["dashboard_id"], "abc-123");
    assert_eq!(out["widget_count"], 2);
}

#[tokio::test]
async fn create_dashboard_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "new-dash-1",
            "title": "New Dashboard",
            "layout_type": "ordered"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::create_dashboard(
        ctx,
        "New Dashboard".into(),
        "ordered".into(),
        vec![json!({"type": "timeseries"})],
        Some("Test dashboard".into()),
    )
    .await
    .unwrap();
    assert_success(&out);
    assert_eq!(out["dashboard_id"], "new-dash-1");
}

#[tokio::test]
async fn delete_dashboard_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/dashboard/old-dash"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"deleted_dashboard_id": "old-dash"})))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::delete_dashboard(ctx, DashboardId("old-dash".into()))
        .await
        .unwrap();
    assert_success(&out);
}

// ============================================================================
// Downtime Tests
// ============================================================================

#[tokio::test]
async fn get_downtimes_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/downtime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"id": 1, "scope": ["env:staging"], "message": "Maintenance"},
            {"id": 2, "scope": ["service:api"], "message": "Deploy"}
        ])))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_downtimes(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_downtimes"], 2);
}

#[tokio::test]
async fn create_downtime_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/downtime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 100,
            "scope": ["env:staging"],
            "message": "Scheduled maintenance"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::create_downtime(
        ctx,
        vec!["env:staging".into()],
        Some(1700000000),
        Some(1700003600),
        Some("Scheduled maintenance".into()),
    )
    .await
    .unwrap();
    assert_success(&out);
    assert_eq!(out["downtime_id"], 100);
}

#[tokio::test]
async fn cancel_downtime_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/downtime/100"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::cancel_downtime(ctx, DowntimeId(100)).await.unwrap();
    assert_success(&out);
}

// ============================================================================
// Synthetics Tests
// ============================================================================

#[tokio::test]
async fn get_synthetics_tests_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/tests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tests": [
                {"public_id": "abc-123", "name": "API Health", "test_type": "api"},
                {"public_id": "def-456", "name": "Browser Test", "test_type": "browser"}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_synthetics_tests(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["test_count"], 2);
}

#[tokio::test]
async fn get_synthetics_locations_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/locations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "locations": [
                {"id": "aws:eu-central-1", "name": "Frankfurt", "is_private": false, "region": {"name": "Europe"}},
                {"id": "aws:us-east-1", "name": "N. Virginia", "is_private": false, "region": {"name": "Americas"}}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_synthetics_locations(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_locations"], 2);
    assert_eq!(out["public_count"], 2);
}

#[tokio::test]
async fn create_synthetics_test_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/synthetics/tests/api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "public_id": "new-test-123",
            "name": "New API Test",
            "type": "api",
            "subtype": "http",
            "config": {
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/health"
                },
                "assertions": []
            },
            "options": {
                "tick_every": 300
            },
            "locations": ["aws:eu-central-1"],
            "status": "live"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::create_synthetics_test(
        ctx,
        "New API Test".into(),
        "api".into(),
        "https://api.example.com/health".into(),
        vec!["aws:eu-central-1".into()],
        Some("Test failed".into()),
        Some(vec!["env:prod".into()]),
        Some(300),
    )
    .await
    .unwrap();
    assert_success(&out);
    assert_eq!(out["public_id"], "new-test-123");
}

#[tokio::test]
async fn create_synthetics_test_invalid_type() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;

    let out = tools::create_synthetics_test(
        ctx,
        "Browser Test".into(),
        "browser".into(), // Not supported
        "https://example.com".into(),
        vec!["aws:eu-central-1".into()],
        None,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"]
        .as_str()
        .unwrap()
        .contains("Only 'api' test type"));
}

#[tokio::test]
async fn trigger_synthetics_tests_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/synthetics/tests/trigger"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "triggered_check_ids": ["check-1", "check-2"],
            "results": []
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::trigger_synthetics_tests(
        ctx,
        vec![SyntheticsTestId("test-1".into()), SyntheticsTestId("test-2".into())],
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "success");
}

#[tokio::test]
async fn trigger_synthetics_tests_empty() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;

    let out = tools::trigger_synthetics_tests(ctx, vec![]).await.unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"]
        .as_str()
        .unwrap()
        .contains("At least one test ID"));
}

// ============================================================================
// Infrastructure Tests
// ============================================================================

#[tokio::test]
async fn get_infrastructure_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/hosts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "host_list": [
                {"name": "host1", "up": true},
                {"name": "host2", "up": true},
                {"name": "host3", "up": false}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_infrastructure(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_hosts"], 3);
    assert_eq!(out["active_hosts"], 2);
}

#[tokio::test]
async fn get_tags_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/tags/hosts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tags": {
                "host1": ["env:prod", "service:api"],
                "host2": ["env:staging"]
            }
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_tags(ctx, None).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["host_count"], 2);
}

// ============================================================================
// Events Tests
// ============================================================================

#[tokio::test]
async fn get_events_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "events": [
                {"id": 1, "title": "Deploy started", "priority": "normal"},
                {"id": 2, "title": "Alert triggered", "priority": "high"}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_events(ctx, 1700000000, 1700003600, None, None)
        .await
        .unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["event_count"], 2);
}

// ============================================================================
// SLO Tests
// ============================================================================

#[tokio::test]
async fn get_slos_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/slo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "slo-1", "name": "API Availability", "target_threshold": 99.9},
                {"id": "slo-2", "name": "Latency SLO", "target_threshold": 95.0}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_slos(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_slos"], 2);
}

// ============================================================================
// Teams and Users Tests
// ============================================================================

#[tokio::test]
async fn get_teams_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/teams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "team-1", "attributes": {"name": "Platform Team"}},
                {"id": "team-2", "attributes": {"name": "SRE Team"}}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_teams(ctx).await.unwrap();
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_teams"], 2);
}

#[tokio::test]
async fn get_users_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "users": [
                {"id": "user-1", "email": "alice@example.com", "status": "Active"},
                {"id": "user-2", "email": "bob@example.com", "status": "Active"}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_users(ctx).await.unwrap();
    assert_success(&out);
    assert_eq!(out["total_users"], 2);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn rate_limited_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limit exceeded"))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_monitors(ctx).await.unwrap();
    assert_eq!(out["status"], "error");
}

#[tokio::test]
async fn server_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = tools::get_dashboards(ctx).await.unwrap();
    assert_eq!(out["status"], "error");
}

// ============================================================================
// Input Validation Tests
// ============================================================================

#[tokio::test]
async fn create_monitor_invalid_type() {
    let server = MockServer::start().await;
    // No mock needed - validation should fail before API call
    let ctx = mock_context(&server).await;
    let out = tools::create_monitor(
        ctx,
        "Test Monitor".into(),
        "invalid_type".into(),
        "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"]
        .as_str()
        .unwrap()
        .contains("Invalid monitor type"));
}

#[tokio::test]
async fn create_monitor_empty_name() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = tools::create_monitor(
        ctx,
        "   ".into(), // Whitespace-only name
        "metric alert".into(),
        "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}

#[tokio::test]
async fn create_monitor_empty_query() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = tools::create_monitor(
        ctx,
        "Test Monitor".into(),
        "metric alert".into(),
        "".into(), // Empty query
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}

#[tokio::test]
async fn create_dashboard_invalid_layout() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = tools::create_dashboard(
        ctx,
        "Test Dashboard".into(),
        "invalid_layout".into(),
        vec![],
        None,
    )
    .await
    .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"]
        .as_str()
        .unwrap()
        .contains("Invalid dashboard layout"));
}

#[tokio::test]
async fn create_dashboard_empty_title() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = tools::create_dashboard(ctx, "  ".into(), "ordered".into(), vec![], None)
        .await
        .unwrap();
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}
