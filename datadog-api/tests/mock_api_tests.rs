use datadog_api::{apis::*, DatadogClient, DatadogConfig};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn create_test_client(server: &MockServer) -> DatadogClient {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
        .with_base_url(server.uri());

    DatadogClient::new(config).unwrap()
}

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
    assert_eq!(dashboards.len(), 1);
    assert_eq!(dashboards[0].id, Some("abc-123".to_string()));
}

#[tokio::test]
async fn test_metrics_api_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "series": [
                {
                    "metric": "system.cpu.user",
                    "pointlist": [[1700000000.0, 50.5], [1700000060.0, 55.2]]
                }
            ]
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
}

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
