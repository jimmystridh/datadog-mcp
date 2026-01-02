//! Comprehensive mock tests for all Datadog API endpoints
//!
//! Tests use wiremock to simulate Datadog API responses and verify
//! correct request formatting and response parsing.

use datadog_api::models::*;
use datadog_api::{apis::*, DatadogClient, DatadogConfig};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// TEST HELPERS
// ============================================================================

async fn create_test_client(server: &MockServer) -> DatadogClient {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
        .with_base_url(server.uri());

    DatadogClient::new(config).unwrap()
}

// ============================================================================
// MONITORS API TESTS
// ============================================================================

#[tokio::test]
async fn test_monitors_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .and(header("dd-api-key", "test_api_key"))
        .and(header("dd-application-key", "test_app_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 12345,
                "name": "Test Monitor",
                "type": "metric alert",
                "query": "avg:system.cpu.user{*} > 80",
                "overall_state": "OK"
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors().await;
    assert!(result.is_ok());

    let monitors = result.unwrap();
    assert_eq!(monitors.len(), 1);
    assert_eq!(monitors[0].id, Some(12345));
    assert_eq!(monitors[0].name, Some("Test Monitor".to_string()));
}

#[tokio::test]
async fn test_monitors_api_list_with_page_size() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .and(query_param("page_size", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors_with_page_size(50).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_monitors_api_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12345,
            "name": "Test Monitor",
            "type": "metric alert",
            "query": "avg:system.cpu.user{*} > 80",
            "overall_state": "Alert"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.get_monitor(12345).await;
    assert!(result.is_ok());

    let monitor = result.unwrap();
    assert_eq!(monitor.id, Some(12345));
    assert_eq!(monitor.overall_state, Some("Alert".to_string()));
}

#[tokio::test]
async fn test_monitors_api_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 99999,
            "name": "New Monitor",
            "type": "metric alert",
            "query": "avg:system.cpu.user{*} > 90",
            "message": "CPU is high!"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let request = MonitorCreateRequest {
        name: "New Monitor".to_string(),
        monitor_type: "metric alert".to_string(),
        query: "avg:system.cpu.user{*} > 90".to_string(),
        message: Some("CPU is high!".to_string()),
        tags: None,
        options: None,
    };

    let result = api.create_monitor(&request).await;
    assert!(result.is_ok());

    let monitor = result.unwrap();
    assert_eq!(monitor.id, Some(99999));
    assert_eq!(monitor.name, Some("New Monitor".to_string()));
}

#[tokio::test]
async fn test_monitors_api_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/monitor/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12345,
            "name": "Updated Monitor",
            "type": "metric alert",
            "query": "avg:system.cpu.user{*} > 95"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let request = MonitorUpdateRequest {
        name: Some("Updated Monitor".to_string()),
        query: Some("avg:system.cpu.user{*} > 95".to_string()),
        message: None,
        tags: None,
        options: None,
    };

    let result = api.update_monitor(12345, &request).await;
    assert!(result.is_ok());

    let monitor = result.unwrap();
    assert_eq!(monitor.name, Some("Updated Monitor".to_string()));
}

#[tokio::test]
async fn test_monitors_api_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/monitor/12345"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted_monitor_id": 12345
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.delete_monitor(12345).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_monitors_api_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/99999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "errors": ["Monitor not found"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.get_monitor(99999).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_not_found());
    assert!(err.is_client_error());
    assert!(!err.is_retryable());
}

#[tokio::test]
async fn test_monitors_api_unauthorized() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "errors": ["Unauthorized"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_unauthorized());
    assert!(!err.is_retryable());
}

#[tokio::test]
async fn test_monitors_api_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "errors": ["Too many requests"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_rate_limited());
    assert!(err.is_retryable());
}

// ============================================================================
// DASHBOARDS API TESTS
// ============================================================================

#[tokio::test]
async fn test_dashboards_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "dashboards": [
                {
                    "id": "abc-123",
                    "title": "Test Dashboard",
                    "layout_type": "ordered"
                },
                {
                    "id": "def-456",
                    "title": "Another Dashboard",
                    "layout_type": "free"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DashboardsApi::new(client);

    let result = api.list_dashboards().await;
    assert!(result.is_ok());

    let list = result.unwrap();
    let dashboards = list.dashboards.unwrap();
    assert_eq!(dashboards.len(), 2);
    assert_eq!(dashboards[0].id, Some("abc-123".to_string()));
}

#[tokio::test]
async fn test_dashboards_api_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard/abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "abc-123",
            "title": "Test Dashboard",
            "layout_type": "ordered",
            "widgets": [
                {"id": 1, "definition": {"type": "note", "content": "Hello"}}
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DashboardsApi::new(client);

    let result = api.get_dashboard("abc-123").await;
    assert!(result.is_ok());

    let dashboard = result.unwrap();
    assert_eq!(dashboard.id, Some("abc-123".to_string()));
    assert_eq!(dashboard.title, Some("Test Dashboard".to_string()));
    assert!(dashboard.widgets.is_some());
}

#[tokio::test]
async fn test_dashboards_api_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dashboard"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "new-dash-123",
            "title": "New Dashboard",
            "layout_type": "ordered"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DashboardsApi::new(client);

    let dashboard = Dashboard {
        id: None,
        title: Some("New Dashboard".to_string()),
        description: None,
        widgets: Some(vec![]),
        layout_type: Some("ordered".to_string()),
        is_read_only: None,
        notify_list: None,
        template_variables: None,
    };

    let result = api.create_dashboard(&dashboard).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.id, Some("new-dash-123".to_string()));
}

#[tokio::test]
async fn test_dashboards_api_update() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PUT"))
        .and(path("/api/v1/dashboard/abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "abc-123",
            "title": "Updated Dashboard",
            "layout_type": "ordered"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DashboardsApi::new(client);

    let dashboard = Dashboard {
        id: Some("abc-123".to_string()),
        title: Some("Updated Dashboard".to_string()),
        description: None,
        widgets: Some(vec![]),
        layout_type: Some("ordered".to_string()),
        is_read_only: None,
        notify_list: None,
        template_variables: None,
    };

    let result = api.update_dashboard("abc-123", &dashboard).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.title, Some("Updated Dashboard".to_string()));
}

#[tokio::test]
async fn test_dashboards_api_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/dashboard/abc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted_dashboard_id": "abc-123"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DashboardsApi::new(client);

    let result = api.delete_dashboard("abc-123").await;
    assert!(result.is_ok());
}

// ============================================================================
// METRICS API TESTS
// ============================================================================

#[tokio::test]
async fn test_metrics_api_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "series": [
                {
                    "metric": "system.cpu.user",
                    "pointlist": [[1700000000.0, 50.5], [1700000060.0, 55.2]],
                    "scope": "host:web-01"
                }
            ],
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MetricsApi::new(client);

    let result = api
        .query_metrics(1700000000, 1700003600, "avg:system.cpu.user{*}")
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let series = response.series.unwrap();
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].metric, Some("system.cpu.user".to_string()));
}

#[tokio::test]
async fn test_metrics_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/metrics"))
        .and(query_param("q", "system.cpu"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "metrics": ["system.cpu.user", "system.cpu.system", "system.cpu.idle"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MetricsApi::new(client);

    let result = api.list_metrics("system.cpu").await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let metrics = response.metrics.unwrap();
    assert_eq!(metrics.len(), 3);
    assert!(metrics.contains(&"system.cpu.user".to_string()));
}

#[tokio::test]
async fn test_metrics_api_get_metadata() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/metrics/system.cpu.user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "description": "Percentage of CPU time spent in user space",
            "short_name": "CPU User",
            "type": "gauge",
            "unit": "percent"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MetricsApi::new(client);

    let result = api.get_metric_metadata("system.cpu.user").await;
    assert!(result.is_ok());

    let metadata = result.unwrap();
    assert_eq!(metadata.metric_type, Some("gauge".to_string()));
    assert_eq!(metadata.unit, Some("percent".to_string()));
}

// ============================================================================
// DOWNTIMES API TESTS
// ============================================================================

#[tokio::test]
async fn test_downtimes_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/downtime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": 1001,
                "scope": ["env:production"],
                "start": 1700000000,
                "end": 1700003600,
                "message": "Scheduled maintenance",
                "active": true
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DowntimesApi::new(client);

    let result = api.list_downtimes().await;
    assert!(result.is_ok());

    let downtimes = result.unwrap();
    assert_eq!(downtimes.len(), 1);
    assert_eq!(downtimes[0].id, Some(1001));
    assert_eq!(downtimes[0].active, Some(true));
}

#[tokio::test]
async fn test_downtimes_api_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/downtime"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 1002,
            "scope": ["env:staging"],
            "start": 1700000000,
            "end": 1700007200,
            "message": "Testing",
            "active": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DowntimesApi::new(client);

    let request = DowntimeCreateRequest {
        scope: vec!["env:staging".to_string()],
        start: Some(1700000000),
        end: Some(1700007200),
        message: Some("Testing".to_string()),
    };

    let result = api.create_downtime(&request).await;
    assert!(result.is_ok());

    let downtime = result.unwrap();
    assert_eq!(downtime.id, Some(1002));
}

#[tokio::test]
async fn test_downtimes_api_cancel() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/downtime/1001"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = DowntimesApi::new(client);

    let result = api.cancel_downtime(1001).await;
    assert!(result.is_ok());
}

// ============================================================================
// EVENTS API TESTS
// ============================================================================

#[tokio::test]
async fn test_events_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "events": [
                {
                    "id": 5001,
                    "title": "Deployment completed",
                    "text": "Version 1.2.3 deployed to production",
                    "date_happened": 1700000000,
                    "tags": ["env:production", "deploy"],
                    "priority": "normal",
                    "alert_type": "info"
                }
            ],
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = EventsApi::new(client);

    let result = api
        .list_events(1699900000, 1700100000, None, None)
        .await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let events = response.events.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].title, Some("Deployment completed".to_string()));
}

#[tokio::test]
async fn test_events_api_list_with_filters() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/events"))
        .and(query_param("priority", "high"))
        .and(query_param("sources", "nagios"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "events": [],
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = EventsApi::new(client);

    let result = api
        .list_events(1699900000, 1700100000, Some("high"), Some("nagios"))
        .await;
    assert!(result.is_ok());
}

// ============================================================================
// LOGS API TESTS
// ============================================================================

#[tokio::test]
async fn test_logs_api_search() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/logs/events/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "log-123",
                    "attributes": {
                        "message": "Error connecting to database",
                        "status": "error",
                        "service": "api-server"
                    }
                }
            ],
            "meta": {
                "page": {
                    "after": "cursor-abc"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = LogsApi::new(client);

    let request = LogsSearchRequest {
        filter: LogsFilter {
            query: "status:error".to_string(),
            from: "now-15m".to_string(),
            to: "now".to_string(),
        },
        page: Some(LogsPage {
            limit: Some(100),
            cursor: None,
        }),
        sort: None,
    };

    let result = api.search_logs(&request).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let logs = response.data.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].id, Some("log-123".to_string()));
}

// ============================================================================
// INFRASTRUCTURE API TESTS
// ============================================================================

#[tokio::test]
async fn test_infrastructure_api_list_hosts() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/hosts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "host_list": [
                {
                    "id": 1,
                    "name": "web-01.example.com",
                    "up": true,
                    "last_reported_time": 1700000000,
                    "meta": {
                        "agent_version": "7.48.0",
                        "cpu_cores": 4,
                        "platform": "linux"
                    }
                }
            ],
            "total_matching": 1,
            "total_returned": 1
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = InfrastructureApi::new(client);

    let result = api.list_hosts().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let hosts = response.host_list.unwrap();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].name, Some("web-01.example.com".to_string()));
    assert_eq!(hosts[0].up, Some(true));
}

#[tokio::test]
async fn test_infrastructure_api_get_tags() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/tags/hosts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tags": {
                "env": ["production", "staging", "development"],
                "service": ["api", "web", "worker"]
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = InfrastructureApi::new(client);

    let result = api.get_tags(None).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let tags = response.tags.unwrap();
    assert!(tags.contains_key("env"));
    assert!(tags.contains_key("service"));
}

#[tokio::test]
async fn test_infrastructure_api_get_tags_with_source() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/tags/hosts"))
        .and(query_param("source", "chef"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tags": {}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = InfrastructureApi::new(client);

    let result = api.get_tags(Some("chef")).await;
    assert!(result.is_ok());
}

// ============================================================================
// SYNTHETICS API TESTS
// ============================================================================

#[tokio::test]
async fn test_synthetics_api_list_tests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/tests"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tests": [
                {
                    "public_id": "abc-def-123",
                    "name": "API Health Check",
                    "type": "api",
                    "status": "live",
                    "tags": ["env:production"]
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SyntheticsApi::new(client);

    let result = api.list_tests().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let tests = response.tests.unwrap();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].public_id, Some("abc-def-123".to_string()));
}

#[tokio::test]
async fn test_synthetics_api_get_test() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/tests/abc-def-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "public_id": "abc-def-123",
            "name": "API Health Check",
            "type": "api",
            "subtype": "http",
            "config": {
                "request": {
                    "method": "GET",
                    "url": "https://api.example.com/health"
                },
                "assertions": [
                    {"type": "statusCode", "operator": "is", "target": 200}
                ]
            },
            "options": {
                "tick_every": 300
            },
            "locations": ["aws:us-east-1"],
            "status": "live"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SyntheticsApi::new(client);

    let result = api.get_test("abc-def-123").await;
    assert!(result.is_ok());

    let test = result.unwrap();
    assert_eq!(test.public_id, "abc-def-123");
    assert_eq!(test.name, "API Health Check");
}

#[tokio::test]
async fn test_synthetics_api_list_locations() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/locations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "locations": [
                {"id": "aws:us-east-1", "name": "N. Virginia (AWS)", "is_private": false},
                {"id": "aws:eu-central-1", "name": "Frankfurt (AWS)", "is_private": false}
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SyntheticsApi::new(client);

    let result = api.list_locations().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.locations.len(), 2);
}

#[tokio::test]
async fn test_synthetics_api_create_test() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/synthetics/tests/api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "public_id": "new-test-123",
            "name": "New API Test",
            "type": "api",
            "subtype": "http",
            "config": {
                "request": {"method": "GET", "url": "https://example.com"},
                "assertions": []
            },
            "options": {"tick_every": 300},
            "locations": ["aws:us-east-1"],
            "status": "paused"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SyntheticsApi::new(client);

    let request = SyntheticsTestCreateRequest {
        name: "New API Test".to_string(),
        test_type: SyntheticsTestType::Api,
        subtype: SyntheticsTestSubtype::Http,
        config: SyntheticsTestConfig {
            request: SyntheticsTestRequest {
                method: "GET".to_string(),
                url: "https://example.com".to_string(),
                timeout: None,
                headers: None,
                body: None,
            },
            assertions: vec![],
        },
        options: SyntheticsTestOptions {
            tick_every: 300,
            min_failure_duration: None,
            min_location_failed: None,
            retry: None,
        },
        locations: vec!["aws:us-east-1".to_string()],
        message: None,
        tags: None,
        status: Some("paused".to_string()),
    };

    let result = api.create_test(&request).await;
    assert!(result.is_ok());

    let test = result.unwrap();
    assert_eq!(test.public_id, "new-test-123");
}

#[tokio::test]
async fn test_synthetics_api_trigger_tests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/synthetics/tests/trigger"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "triggered_check_ids": ["abc-123"],
            "results": [
                {"public_id": "abc-123", "result_id": "result-456"}
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SyntheticsApi::new(client);

    let result = api.trigger_tests(vec!["abc-123".to_string()]).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.triggered_check_ids.len(), 1);
}

// ============================================================================
// INCIDENTS API TESTS
// ============================================================================

#[tokio::test]
async fn test_incidents_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/incidents"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "incident-001",
                    "attributes": {
                        "title": "Database outage",
                        "state": "active",
                        "created": "2024-01-15T10:00:00Z",
                        "modified": "2024-01-15T10:30:00Z"
                    }
                }
            ],
            "meta": {
                "pagination": {
                    "next_offset": 10,
                    "size": 10
                }
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = IncidentsApi::new(client);

    let result = api.list_incidents(Some(10)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let incidents = response.data.unwrap();
    assert_eq!(incidents.len(), 1);
    assert_eq!(incidents[0].id, Some("incident-001".to_string()));
}

// ============================================================================
// SLOS API TESTS
// ============================================================================

#[tokio::test]
async fn test_slos_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/slo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "slo-001",
                    "name": "API Availability",
                    "description": "99.9% uptime",
                    "tags": ["service:api"],
                    "thresholds": [
                        {"target": 99.9, "timeframe": "30d"}
                    ]
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SLOsApi::new(client);

    let result = api.list_slos().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let slos = response.data.unwrap();
    assert_eq!(slos.len(), 1);
    assert_eq!(slos[0].name, Some("API Availability".to_string()));
}

// ============================================================================
// NOTEBOOKS API TESTS
// ============================================================================

#[tokio::test]
async fn test_notebooks_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/notebooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": 12345,
                    "attributes": {
                        "name": "Investigation Notes",
                        "created": "2024-01-10T08:00:00Z",
                        "modified": "2024-01-15T14:30:00Z"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = NotebooksApi::new(client);

    let result = api.list_notebooks().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let notebooks = response.data.unwrap();
    assert_eq!(notebooks.len(), 1);
    assert_eq!(notebooks[0].id, Some(12345));
}

// ============================================================================
// SECURITY API TESTS
// ============================================================================

#[tokio::test]
async fn test_security_api_list_rules() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/security_monitoring/rules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "rule-001",
                    "attributes": {
                        "name": "SSH Brute Force Detection",
                        "isEnabled": true,
                        "message": "Multiple failed SSH attempts detected"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = SecurityApi::new(client);

    let result = api.list_security_rules().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let rules = response.data.unwrap();
    assert_eq!(rules.len(), 1);
}

// ============================================================================
// TEAMS API TESTS
// ============================================================================

#[tokio::test]
async fn test_teams_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/teams"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "team-001",
                    "attributes": {
                        "name": "Platform Engineering",
                        "handle": "platform-eng"
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TeamsApi::new(client);

    let result = api.list_teams().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let teams = response.data.unwrap();
    assert_eq!(teams.len(), 1);
}

// ============================================================================
// USERS API TESTS
// ============================================================================

#[tokio::test]
async fn test_users_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "users": [
                {
                    "id": "user-001",
                    "name": "John Doe",
                    "email": "john@example.com",
                    "handle": "john.doe",
                    "verified": true
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = UsersApi::new(client);

    let result = api.list_users().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let users = response.users.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].email, Some("john@example.com".to_string()));
}

// ============================================================================
// TRACES API TESTS
// ============================================================================

#[tokio::test]
async fn test_traces_api_send() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v0.4/traces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TracesApi::new(client);

    let span = Span {
        span_id: 123456789,
        trace_id: 987654321,
        parent_id: 0,
        service: "test-service".to_string(),
        resource: "/api/test".to_string(),
        name: "web.request".to_string(),
        start: 1700000000000000000,
        duration: 50000000,
        error: 0,
        meta: std::collections::HashMap::new(),
        metrics: std::collections::HashMap::new(),
        span_type: Some("web".to_string()),
    };

    let result = api.send_traces(vec![vec![span]]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_traces_api_search() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/traces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "trace_id": "abc123",
                    "spans": []
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TracesApi::new(client);

    let query = TraceQuery {
        service: Some("api".to_string()),
        operation: None,
        resource: None,
        start: 1700000000,
        end: 1700003600,
        limit: Some(10),
    };

    let result = api.search_traces(&query).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_traces_api_list_services() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/apm/services"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            "api-service",
            "web-service",
            "database"
        ])))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TracesApi::new(client);

    let result = api.list_services(1700000000, 1700003600).await;
    assert!(result.is_ok());

    let services = result.unwrap();
    assert!(services.contains(&"api-service".to_string()));
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_server_error_is_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({
            "errors": ["Service Unavailable"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_server_error());
    assert!(err.is_retryable());
    assert_eq!(err.status_code(), Some(503));
}

#[tokio::test]
async fn test_forbidden_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "errors": ["Forbidden: insufficient permissions"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let result = api.list_monitors().await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_client_error());
    assert!(!err.is_retryable());
    assert_eq!(err.status_code(), Some(403));
}

#[tokio::test]
async fn test_bad_request_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "errors": ["Invalid query syntax"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);

    let request = MonitorCreateRequest {
        name: "Test".to_string(),
        monitor_type: "metric alert".to_string(),
        query: "invalid query".to_string(),
        message: None,
        tags: None,
        options: None,
    };

    let result = api.create_monitor(&request).await;
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_client_error());
    assert_eq!(err.status_code(), Some(400));
}
