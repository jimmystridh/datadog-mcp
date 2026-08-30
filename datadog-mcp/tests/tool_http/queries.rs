use super::*;

#[tokio::test]
async fn get_metrics_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/query/timeseries"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "attributes": {
                    "series": [{"group_tags": ["host:local"], "query_index": 0}],
                    "times": [1000, 2000],
                    "values": [[2.0, 3.0]]
                },
                "type": "timeseries_response"
            }
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(ctx, tools::get_metrics(ctx.clone(), "test".into(), 0, 10));
    assert_eq!(out["status"], "success");
    assert!(out["filepath"].is_null());
    assert!(out["data"].is_object());
    assert_eq!(out["series_count"], 1);
    assert_eq!(out["data_points"], 2);
}

#[tokio::test]
async fn get_metrics_unauthorized() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v2/query/timeseries"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(ctx, tools::get_metrics(ctx.clone(), "test".into(), 0, 10));
    assert_eq!(out["status"], "error");
}

#[tokio::test]
async fn search_metrics_filters_active_metrics_since_requested_timestamp() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/metrics"))
        .and(query_param("from", "1700000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "metrics": [
                "system.cpu.user",
                "sqlserver.database.state",
                "SQLServer.queries.count"
            ]
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::search_metrics(ctx.clone(), "sqlserver".into(), Some(1_700_000_000))
    );

    assert_eq!(out["status"], "success");
    assert_eq!(out["from_timestamp"], 1_700_000_000);
    assert_eq!(out["metric_count"], 2);
    assert_eq!(
        out["sample_metrics"],
        json!(["sqlserver.database.state", "SQLServer.queries.count"])
    );
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
    let out = call_tool!(ctx, tools::get_monitor(ctx.clone(), MonitorId(42)));
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
    let out = call_tool!(
        ctx,
        tools::search_logs(
            ctx.clone(),
            "env:prod".into(),
            "now-1h".into(),
            "now".into(),
            Some(10),
            None,
        )
    );
    assert_eq!(out["status"], "success");
    assert_eq!(out["log_count"], 2);
}

#[tokio::test]
async fn update_dashboard_preserves_unknown_fields_and_widgets() {
    let server = MockServer::start().await;
    let existing = json!({
        "id": "dash-1",
        "title": "Before",
        "layout_type": "ordered",
        "widgets": [{
            "definition": {
                "type": "geomap",
                "requests": [{"custom": "preserve-me"}]
            }
        }],
        "custom_top_level": {"future_field": true}
    });
    Mock::given(method("GET"))
        .and(path("/api/v1/dashboard/dash-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(existing))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/dashboard/dash-1"))
        .and(body_json(json!({
            "title": "After",
            "layout_type": "ordered",
            "widgets": [{
                "definition": {
                    "type": "geomap",
                    "requests": [{"custom": "preserve-me"}]
                }
            }],
            "custom_top_level": {"future_field": true}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "dash-1",
            "title": "After",
            "layout_type": "ordered"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::update_dashboard(
            ctx.clone(),
            DashboardId("dash-1".into()),
            Some("After".into()),
            None,
        )
    );
    assert_eq!(out["status"], "success");
    assert_eq!(out["operation_status"], "updated");
}

#[tokio::test]
async fn update_synthetics_preserves_browser_specific_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/synthetics/tests/browser-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "public_id": "browser-1",
            "name": "Before",
            "type": "browser",
            "config": {"assertions": [], "future_config": true},
            "options": {"tick_every": 300, "future_option": "keep"},
            "locations": ["aws:eu-central-1"],
            "steps": [{"type": "goToUrl", "params": {"value": "https://example.com"}}],
            "status": "live"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/synthetics/tests/browser-1"))
        .and(body_json(json!({
            "name": "After",
            "type": "browser",
            "config": {"assertions": [], "future_config": true},
            "options": {"tick_every": 300, "future_option": "keep"},
            "locations": ["aws:eu-central-1"],
            "steps": [{"type": "goToUrl", "params": {"value": "https://example.com"}}],
            "status": "live"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "public_id": "browser-1",
            "name": "After",
            "status": "live"
        })))
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::update_synthetics_test(
            ctx.clone(),
            SyntheticsTestId("browser-1".into()),
            Some("After".into()),
            None,
            None,
            None,
            None,
            None,
        )
    );
    assert_eq!(out["status"], "success");
    assert_eq!(out["test_status"], "live");
}

// ============================================================================
