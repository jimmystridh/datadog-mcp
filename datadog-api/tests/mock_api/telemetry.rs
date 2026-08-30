use super::*;

// METRICS API TESTS
// ============================================================================

#[tokio::test]
async fn test_metrics_api_query() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/query/timeseries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "attributes": {
                    "series": [{
                        "group_tags": ["host:web-01"],
                        "query_index": 0,
                        "unit": null
                    }],
                    "times": [1700000000000_i64, 1700000060000_i64],
                    "values": [[50.5, 55.2]]
                },
                "type": "timeseries_response"
            }
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
    let attributes = response.data.unwrap().attributes;
    assert_eq!(attributes.series.len(), 1);
    assert_eq!(attributes.series[0].group_tags, ["host:web-01"]);
    assert_eq!(attributes.values[0], [Some(50.5), Some(55.2)]);
}

#[tokio::test]
async fn test_metrics_api_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/metrics"))
        .and(query_param("from", "1700000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "metrics": ["system.cpu.user", "system.cpu.system", "system.cpu.idle"]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = MetricsApi::new(client);

    let result = api.list_active_metrics(1700000000).await;
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

    let result = api.list_events(1699900000, 1700100000, None, None).await;
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
