use super::*;
use datadog_mcp::tool_inputs::{GetDatabaseQueryMetricsInput, SearchDatabaseQuerySamplesInput};

#[tokio::test]
async fn database_samples_return_full_details_without_disk_cache() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/logs-analytics/list"))
        .and(query_param("type", "databasequery"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"result":{"events":[{
                "event":{"custom":{"db":{"statement":"SELECT 1"},"sqlserver":{"session_id":"85"}}}
            }]}})),
        )
        .mount(&server)
        .await;
    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::search_database_query_samples(
            ctx.clone(),
            SearchDatabaseQuerySamplesInput {
                query: "env:prod".into(),
                from_timestamp: 0,
                to_timestamp: 1,
                sample_type: None,
                limit: Some(1),
            }
        )
    );
    assert_success(&out);
    assert_eq!(out["sample_count"], 1);
    assert_eq!(out["sample_type"], "activity");
    assert_eq!(out["possibly_truncated"], true);
    assert_eq!(
        out["data"]["result"]["events"][0]["event"]["custom"]["sqlserver"]["session_id"],
        "85"
    );
    assert!(out["filepath"].is_null());
}

#[tokio::test]
async fn database_query_metrics_preserve_aligned_columns_and_defaults() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/api/v2/query/scalar"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":{"attributes":{"columns":[
            {"type":"group","values":[["query_signature:abc"]]}, {"type":"number","values":[1.0]}
        ]}}})))
        .mount(&server).await;
    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::get_database_query_metrics(
            ctx.clone(),
            GetDatabaseQueryMetricsInput {
                query: "sum:sqlserver.queries.time{*} by {query_signature}".into(),
                from_timestamp: 0,
                to_timestamp: 1,
                aggregator: None,
                limit: None,
            }
        )
    );
    assert_success(&out);
    assert_eq!(out["aggregator"], "sum");
    assert_eq!(out["limit"], 100);
    assert_eq!(
        out["data"]["data"]["attributes"]["columns"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(out["filepath"].is_null());
}

#[tokio::test]
async fn database_invalid_time_is_an_mcp_error() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let result = render_tool_result(
        tools::search_database_query_samples(
            ctx.clone(),
            SearchDatabaseQuerySamplesInput {
                query: "".into(),
                from_timestamp: 2,
                to_timestamp: 1,
                sample_type: None,
                limit: None,
            },
        )
        .await,
        &ctx,
    )
    .await;
    assert_eq!(result.is_error, Some(true));
    assert!(server.received_requests().await.unwrap().is_empty());
}
