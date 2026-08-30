use super::*;

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
    let out = call_tool!(ctx, tools::get_synthetics_tests(ctx.clone()));
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
    let out = call_tool!(ctx, tools::get_synthetics_locations(ctx.clone()));
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
    let out = call_tool!(
        ctx,
        tools::create_synthetics_test(
            ctx.clone(),
            "New API Test".into(),
            "api".into(),
            "https://api.example.com/health".into(),
            vec!["aws:eu-central-1".into()],
            Some("Test failed".into()),
            Some(vec!["env:prod".into()]),
            Some(300),
        )
    );
    assert_success(&out);
    assert_eq!(out["public_id"], "new-test-123");
}

#[tokio::test]
async fn create_synthetics_test_invalid_type() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;

    let out = call_tool!(
        ctx,
        tools::create_synthetics_test(
            ctx.clone(),
            "Browser Test".into(),
            "browser".into(), // Not supported
            "https://example.com".into(),
            vec!["aws:eu-central-1".into()],
            None,
            None,
            None,
        )
    );
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
    let out = call_tool!(
        ctx,
        tools::trigger_synthetics_tests(
            ctx.clone(),
            vec![
                SyntheticsTestId("test-1".into()),
                SyntheticsTestId("test-2".into()),
            ],
        )
    );
    assert_eq!(out["status"], "success");
}

#[tokio::test]
async fn trigger_synthetics_tests_empty() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;

    let out = call_tool!(ctx, tools::trigger_synthetics_tests(ctx.clone(), vec![]));
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
    let out = call_tool!(ctx, tools::get_infrastructure(ctx.clone()));
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
    let out = call_tool!(ctx, tools::get_tags(ctx.clone(), None));
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
    let out = call_tool!(
        ctx,
        tools::get_events(ctx.clone(), 1700000000, 1700003600, None, None)
    );
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
    let out = call_tool!(ctx, tools::get_slos(ctx.clone()));
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
        .and(path("/api/v2/team"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "team-1", "attributes": {"name": "Platform Team"}},
                {"id": "team-2", "attributes": {"name": "SRE Team"}}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(ctx, tools::get_teams(ctx.clone(), None, None));
    assert_eq!(out["status"], "success");
    assert_eq!(out["total_teams"], 2);
}

#[tokio::test]
async fn get_users_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v2/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "user-1", "attributes": {"email": "alice@example.com", "status": "Active"}},
                {"id": "user-2", "attributes": {"email": "bob@example.com", "status": "Active"}}
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(ctx, tools::get_users(ctx.clone(), None, None));
    assert_success(&out);
    assert_eq!(out["total_users"], 2);
}

// ============================================================================
