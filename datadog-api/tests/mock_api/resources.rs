use super::*;

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
async fn test_monitors_api_get_with_group_states_and_downtimes() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/12345"))
        .and(query_param("group_states", "all"))
        .and(query_param("with_downtimes", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12345,
            "name": "Grouped Monitor",
            "type": "service check",
            "overall_state": "Alert",
            "options": {
                "notify_no_data": false,
                "timeout_h": null,
                "notify_by": ["env", "database_instance"]
            },
            "state": {
                "groups": {
                    "env:prod,database_instance:old": {
                        "name": "env:prod,database_instance:old",
                        "status": "Alert",
                        "last_triggered_ts": 1788301551
                    },
                    "env:prod,database_instance:new": {
                        "name": "env:prod,database_instance:new",
                        "status": "OK"
                    }
                }
            },
            "matching_downtimes": [
                {"id": 999, "scope": ["env:prod,database_instance:old"]}
            ],
            "draft_status": "published"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MonitorsApi::new(client);
    let options = GetMonitorOptions {
        group_states: Some("all"),
        with_downtimes: Some(true),
    };

    let monitor = api.get_monitor_with_options(12345, &options).await.unwrap();

    let groups = monitor.state.unwrap().groups.unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(
        groups["env:prod,database_instance:old"].status.as_deref(),
        Some("Alert")
    );
    assert_eq!(monitor.matching_downtimes.unwrap()[0]["id"], 999);
    assert_eq!(
        monitor.options.unwrap().notify_by,
        Some(vec!["env".to_string(), "database_instance".to_string()])
    );
    assert_eq!(monitor.additional_properties["draft_status"], "published");
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
    assert_eq!(dashboard.id(), Some(&serde_json::json!("abc-123")));
    assert_eq!(dashboard.title(), Some("Test Dashboard"));
    assert_eq!(dashboard.widget_count(), 1);
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

    let dashboard = DashboardDocument::new(
        "New Dashboard".to_string(),
        "ordered".to_string(),
        vec![],
        None,
    );

    let result = api.create_dashboard(&dashboard).await;
    assert!(result.is_ok());

    let created = result.unwrap();
    assert_eq!(created.id(), Some(&serde_json::json!("new-dash-123")));
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

    let dashboard = DashboardDocument::new(
        "Updated Dashboard".to_string(),
        "ordered".to_string(),
        vec![],
        None,
    );

    let result = api.update_dashboard("abc-123", &dashboard).await;
    assert!(result.is_ok());

    let updated = result.unwrap();
    assert_eq!(updated.title(), Some("Updated Dashboard"));
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
