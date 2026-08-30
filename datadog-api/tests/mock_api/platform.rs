use super::*;

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
    assert_eq!(test.public_id(), Some(&serde_json::json!("abc-def-123")));
    assert_eq!(test.name(), Some("API Health Check"));
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

    let result = api.list_incidents(OffsetPage::new(10, None).unwrap()).await;
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
        .and(path("/api/v2/team"))
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

    let result = api.list_teams(NumberedPage::default()).await;
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
        .and(path("/api/v2/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": [
                {
                    "id": "user-001",
                    "attributes": {
                        "name": "John Doe",
                        "email": "john@example.com",
                        "handle": "john.doe",
                        "verified": true
                    }
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server).await;
    let api = UsersApi::new(client);

    let result = api.list_users(NumberedPage::default()).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    let users = response.data.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(
        users[0].attributes.as_ref().unwrap()["email"],
        "john@example.com"
    );
}

// ============================================================================
