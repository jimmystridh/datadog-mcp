use super::*;

// TRACES API TESTS
// ============================================================================

#[tokio::test]
async fn test_spans_api_search() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v2/spans/events/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [{"id": "span-1", "type": "spans", "attributes": {}}],
            "meta": {"page": {"after": "next"}}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TracesApi::new(client);

    let request = SpansSearchRequest {
        data: SpansSearchData {
            attributes: SpansSearchRequestAttributes {
                filter: SpansSearchFilter {
                    from: "now-15m".to_string(),
                    query: "service:test-service".to_string(),
                    to: "now".to_string(),
                },
                page: Some(SpansSearchPage {
                    cursor: None,
                    limit: Some(10),
                }),
                sort: Some("timestamp".to_string()),
            },
            resource_type: "search_request".to_string(),
        },
    };

    let result = api.search_spans(&request).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().data.unwrap().len(), 1);
}

#[tokio::test]
async fn test_traces_api_list_services() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v2/apm/services"))
        .and(query_param("filter[env]", "prod"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "type": "services_list",
                "attributes": {"services": ["api-service", "web-service", "database"]}
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = TracesApi::new(client);

    let result = api.list_services("prod").await;
    assert!(result.is_ok());

    let services = result.unwrap();
    assert_eq!(services.data["attributes"]["services"][0], "api-service");
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
async fn read_requests_retry_but_write_requests_do_not() {
    let read_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/retry-read"))
        .respond_with(ResponseTemplate::new(503).set_body_string("unavailable"))
        .mount(&read_server)
        .await;
    let mut read_config = DatadogConfig::new("test_api_key".into(), "test_app_key".into())
        .with_base_url(read_server.uri());
    read_config.retry_config.max_retries = 2;
    read_config.retry_config.initial_backoff_ms = 1;
    read_config.retry_config.max_backoff_ms = 2;
    let read_client = DatadogClient::new(read_config).unwrap();
    let _: datadog_api::Result<serde_json::Value> = read_client.get("/retry-read").await;
    assert_eq!(read_server.received_requests().await.unwrap().len(), 3);

    let write_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/write-once"))
        .respond_with(ResponseTemplate::new(503).set_body_string("unavailable"))
        .mount(&write_server)
        .await;
    let mut write_config = DatadogConfig::new("test_api_key".into(), "test_app_key".into())
        .with_base_url(write_server.uri());
    write_config.retry_config.max_retries = 2;
    let write_client = DatadogClient::new(write_config).unwrap();
    let _: datadog_api::Result<serde_json::Value> = write_client
        .post("/write-once", &serde_json::json!({}))
        .await;
    assert_eq!(write_server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn response_bodies_are_bounded_before_deserialization() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/oversized"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": "x".repeat(128)
        })))
        .mount(&mock_server)
        .await;

    let mut config = DatadogConfig::new("test_api_key".into(), "test_app_key".into())
        .with_base_url(mock_server.uri());
    config.http_config.max_response_bytes = 32;
    let client = DatadogClient::new(config).unwrap();

    let result: datadog_api::Result<serde_json::Value> = client.get("/oversized").await;
    assert!(matches!(
        result,
        Err(datadog_api::Error::ResponseTooLarge { limit: 32, .. })
    ));
}

#[tokio::test]
async fn total_deadline_includes_response_body_streaming() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 2048];
        let _ = stream.read(&mut request);
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        stream.flush().unwrap();
        std::thread::sleep(Duration::from_secs(2));
        let _ = stream.write_all(b"{}");
    });

    let mut config = DatadogConfig::new("test_api_key".into(), "test_app_key".into())
        .with_base_url(format!("http://{address}"));
    config.http_config.timeout_secs = 10;
    config.http_config.total_timeout_secs = 1;
    config.retry_config.max_retries = 0;
    let client = DatadogClient::new(config).unwrap();

    let started = Instant::now();
    let result: datadog_api::Result<serde_json::Value> = client.get("/slow-body").await;

    assert!(matches!(
        result,
        Err(datadog_api::Error::RequestDeadlineExceeded(1))
    ));
    assert!(started.elapsed() < Duration::from_millis(1_750));
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
