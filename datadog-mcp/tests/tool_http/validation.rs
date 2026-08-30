use super::*;

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
    let out = call_tool!(ctx, tools::get_monitors(ctx.clone()));
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
    let out = call_tool!(ctx, tools::get_dashboards(ctx.clone()));
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
    let out = call_tool!(
        ctx,
        tools::create_monitor(
            ctx.clone(),
            "Test Monitor".into(),
            "invalid_type".into(),
            "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
            None,
            None,
            None,
        )
    );
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
    let out = call_tool!(
        ctx,
        tools::create_monitor(
            ctx.clone(),
            "   ".into(), // Whitespace-only name
            "metric alert".into(),
            "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
            None,
            None,
            None,
        )
    );
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}

#[tokio::test]
async fn create_monitor_empty_query() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::create_monitor(
            ctx.clone(),
            "Test Monitor".into(),
            "metric alert".into(),
            "".into(), // Empty query
            None,
            None,
            None,
        )
    );
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}

#[tokio::test]
async fn create_dashboard_invalid_layout() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::create_dashboard(
            ctx.clone(),
            "Test Dashboard".into(),
            "invalid_layout".into(),
            vec![],
            None,
        )
    );
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
    let out = call_tool!(
        ctx,
        tools::create_dashboard(ctx.clone(), "  ".into(), "ordered".into(), vec![], None)
    );
    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("Empty"));
}
