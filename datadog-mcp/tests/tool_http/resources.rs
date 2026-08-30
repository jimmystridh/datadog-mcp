use super::*;

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
    let out = call_tool!(ctx, tools::get_monitors(ctx.clone()));
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
    let out = call_tool!(ctx, tools::get_monitor(ctx.clone(), MonitorId(123)));
    assert_success(&out);
    assert_eq!(out["monitor_id"], 123);
    assert_eq!(out["monitor_name"], "Test Monitor");
    assert_eq!(out["monitor_status"], "ok");
}

#[tokio::test]
async fn get_monitor_preserves_no_data_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor/124"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 124,
            "name": "New Monitor",
            "overall_state": "No Data",
            "monitor_type": "metric alert",
            "query": "avg(last_5m):avg:system.cpu.user{*} > 80"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(ctx, tools::get_monitor(ctx.clone(), MonitorId(124)));

    assert_eq!(out["status"], "success");
    assert_eq!(out["monitor_status"], "no_data");
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
    let out = call_tool!(
        ctx,
        tools::create_monitor(
            ctx.clone(),
            "New Monitor".into(),
            "metric alert".into(),
            "avg(last_5m):avg:system.cpu.user{*} > 80".into(),
            Some("Alert message".into()),
            None,
            None,
        )
    );
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
    let out = call_tool!(ctx, tools::delete_monitor(ctx.clone(), MonitorId(789)));
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
    let out = call_tool!(ctx, tools::get_dashboards(ctx.clone()));
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
    let out = call_tool!(
        ctx,
        tools::get_dashboard(ctx.clone(), DashboardId("abc-123".into()))
    );
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
    let out = call_tool!(
        ctx,
        tools::create_dashboard(
            ctx.clone(),
            "New Dashboard".into(),
            "ordered".into(),
            vec![json!({"type": "timeseries"})],
            Some("Test dashboard".into()),
        )
    );
    assert_success(&out);
    assert_eq!(out["dashboard_id"], "new-dash-1");
}

#[tokio::test]
async fn delete_dashboard_success() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/dashboard/old-dash"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"deleted_dashboard_id": "old-dash"})),
        )
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::delete_dashboard(ctx.clone(), DashboardId("old-dash".into()))
    );
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
    let out = call_tool!(ctx, tools::get_downtimes(ctx.clone()));
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
    let out = call_tool!(
        ctx,
        tools::create_downtime(
            ctx.clone(),
            vec!["env:staging".into()],
            Some(1700000000),
            Some(1700003600),
            Some("Scheduled maintenance".into()),
        )
    );
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
    let out = call_tool!(ctx, tools::cancel_downtime(ctx.clone(), DowntimeId(100)));
    assert_success(&out);
}

// ============================================================================
