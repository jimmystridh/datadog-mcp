use super::*;
use datadog_api::apis::database_monitoring::{DatabaseSampleType, QueryAggregator};

#[tokio::test]
async fn database_samples_use_dbm_index_and_preserve_session_attributes() {
    let server = MockServer::start().await;
    let response = serde_json::json!({"status":"done", "result":{"events":[{
        "event":{"custom":{"db":{"statement":"SELECT 1", "wait_event":"XE_LIVE_TARGET_TVF"},
            "sqlserver":{"session_id":"85", "unknown_future_attribute":42}}}
    }], "paging":{"after":"opaque"}}});
    Mock::given(method("POST"))
        .and(path("/api/v1/logs-analytics/list"))
        .and(query_param("type", "databasequery"))
        .and(header("DD-API-KEY", "test_api_key"))
        .and(header("DD-APPLICATION-KEY", "test_app_key"))
        .and(body_json(serde_json::json!({"list":{
            "indexes":["databasequery"], "limit":25,
            "search":{"query":"(env:prod @sqlserver.session_id:85) AND dbm_type:activity"},
            "sorts":[{"time":{"order":"desc"}}],
            "time":{"from":1700000000000_i64,"to":1700000600000_i64}
        }})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .expect(1)
        .mount(&server)
        .await;
    let api = DatabaseMonitoringApi::new(create_test_client(&server).await);
    let result = api
        .search_samples(
            "env:prod @sqlserver.session_id:85",
            DatabaseSampleType::Activity,
            1700000000,
            1700000600,
            25,
        )
        .await
        .unwrap();
    assert_eq!(result, response);
}

#[tokio::test]
async fn database_plan_samples_and_permission_errors() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/logs-analytics/list"))
        .and(body_json(serde_json::json!({"list":{
            "indexes":["databasequery"], "limit":1, "search":{"query":"dbm_type:plan"},
            "sorts":[{"time":{"order":"desc"}}], "time":{"from":0,"to":1000}
        }})))
        .respond_with(
            ResponseTemplate::new(403).set_body_json(serde_json::json!({"errors":["Forbidden"]})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let api = DatabaseMonitoringApi::new(create_test_client(&server).await);
    let error = api
        .search_samples("", DatabaseSampleType::Plan, 0, 1, 1)
        .await
        .unwrap_err();
    assert_eq!(error.status_code(), Some(403));
    assert!(error.to_string().contains("unscoped application key"));
}

#[tokio::test]
async fn database_query_metrics_rank_signatures_with_scalar_api() {
    let server = MockServer::start().await;
    let query = "sum:sqlserver.queries.time{env:prod} by {query_signature,database_instance}";
    let response = serde_json::json!({"data":{"type":"scalar_response","attributes":{"columns":[
        {"type":"group","values":[["query_signature:abc","database_instance:prod"]]},
        {"type":"number","values":[123.0]}
    ]}}});
    Mock::given(method("POST")).and(path("/api/v2/query/scalar"))
        .and(body_json(serde_json::json!({"data":{"type":"scalar_request","attributes":{
            "from":1700000000000_i64,"to":1700000600000_i64,
            "formulas":[{"formula":"query1","limit":{"count":10,"order":"desc"}}],
            "queries":[{"name":"query1","data_source":"metrics","query":query,"aggregator":"sum"}]
        }}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .expect(1).mount(&server).await;
    let api = DatabaseMonitoringApi::new(create_test_client(&server).await);
    assert_eq!(
        api.query_metrics(query, QueryAggregator::Sum, 1700000000, 1700000600, 10)
            .await
            .unwrap(),
        response
    );
}

#[tokio::test]
async fn database_queries_reject_invalid_input_before_network() {
    let server = MockServer::start().await;
    let api = DatabaseMonitoringApi::new(create_test_client(&server).await);
    for (from, to, limit) in [
        (-1, 1, 1),
        (2, 1, 1),
        (1, 1, 1),
        (0, i64::MAX, 1),
        (0, 1, 0),
        (0, 1, 1001),
    ] {
        assert!(api
            .search_samples("", DatabaseSampleType::Activity, from, to, limit)
            .await
            .is_err());
        assert!(api
            .query_metrics(
                "sum:sqlserver.queries.time{*}",
                QueryAggregator::Sum,
                from,
                to,
                limit
            )
            .await
            .is_err());
    }
    assert!(api
        .query_metrics(" ", QueryAggregator::Sum, 0, 1, 1)
        .await
        .is_err());
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn database_samples_distinguish_empty_results_from_malformed_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/logs-analytics/list"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"result":{"events":[]}})),
        )
        .mount(&server)
        .await;
    let api = DatabaseMonitoringApi::new(create_test_client(&server).await);
    assert!(api
        .search_samples("", DatabaseSampleType::Activity, 0, 1, 1)
        .await
        .is_ok());
    server.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"result":{}})))
        .mount(&server)
        .await;
    assert!(matches!(
        api.search_samples("", DatabaseSampleType::Activity, 0, 1, 1)
            .await,
        Err(datadog_api::Error::InvalidResponse(_))
    ));
    assert!(matches!(
        api.query_metrics(
            "sum:sqlserver.queries.time{*}",
            QueryAggregator::Sum,
            0,
            1,
            1
        )
        .await,
        Err(datadog_api::Error::InvalidResponse(_))
    ));
}

#[test]
fn database_app_origin_matches_region_and_mock_override() {
    for (site, origin) in [
        ("datadoghq.eu", "https://app.datadoghq.eu"),
        ("datadoghq.com", "https://app.datadoghq.com"),
        ("us3.datadoghq.com", "https://us3.datadoghq.com"),
        ("ap1.datadoghq.com", "https://ap1.datadoghq.com"),
        ("ddog-gov.com", "https://app.ddog-gov.com"),
        ("us2.ddog-gov.com", "https://us2.ddog-gov.com"),
    ] {
        assert_eq!(
            DatadogConfig::new("k".into(), "a".into())
                .with_site(site.into())
                .app_base_url(),
            origin
        );
    }
    assert_eq!(
        DatadogConfig::new("k".into(), "a".into())
            .with_base_url("http://127.0.0.1:1234".into())
            .app_base_url(),
        "http://127.0.0.1:1234"
    );
}
