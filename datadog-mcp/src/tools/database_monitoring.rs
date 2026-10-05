//! Database Monitoring tools using API-key authentication.

use crate::response::ToolOutput;
use crate::state::ToolContext;
use crate::tool_inputs::{
    DatabaseQueryAggregator, DatabaseSampleType, GetDatabaseQueryMetricsInput,
    SearchDatabaseQuerySamplesInput,
};
use datadog_api::apis::database_monitoring::{
    DatabaseSampleType as ApiSampleType, QueryAggregator,
};
use serde_json::json;

pub async fn search_database_query_samples(
    ctx: ToolContext,
    input: SearchDatabaseQuerySamplesInput,
) -> anyhow::Result<ToolOutput> {
    let sample_type = match input.sample_type.unwrap_or(DatabaseSampleType::Activity) {
        DatabaseSampleType::Activity => ApiSampleType::Activity,
        DatabaseSampleType::Plan => ApiSampleType::Plan,
    };
    let limit = input.limit.unwrap_or(100);
    let result = ctx
        .database_monitoring_api()
        .search_samples(
            &input.query,
            sample_type,
            input.from_timestamp,
            input.to_timestamp,
            limit,
        )
        .await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        format!(
            "Retrieved {} database {} samples",
            data["result"]["events"].as_array().map_or(0, Vec::len),
            sample_type.as_str()
        ),
        {
            json!({
                "sample_count": data["result"]["events"].as_array().map_or(0, Vec::len),
                "sample_type": sample_type.as_str(),
                "limit": limit,
                "possibly_truncated": data["result"]["events"].as_array().is_some_and(|events| events.len() >= limit as usize),
                "pagination": "One bounded page, newest first. Narrow query or time range for additional samples; raw result metadata is preserved.",
            })
        }
    )
}

pub async fn get_database_query_metrics(
    ctx: ToolContext,
    input: GetDatabaseQueryMetricsInput,
) -> anyhow::Result<ToolOutput> {
    let aggregator = match input.aggregator.unwrap_or(DatabaseQueryAggregator::Sum) {
        DatabaseQueryAggregator::Sum => QueryAggregator::Sum,
        DatabaseQueryAggregator::Avg => QueryAggregator::Avg,
        DatabaseQueryAggregator::Min => QueryAggregator::Min,
        DatabaseQueryAggregator::Max => QueryAggregator::Max,
        DatabaseQueryAggregator::Last => QueryAggregator::Last,
    };
    let result = ctx
        .database_monitoring_api()
        .query_metrics(
            &input.query,
            aggregator,
            input.from_timestamp,
            input.to_timestamp,
            input.limit.unwrap_or(100),
        )
        .await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        "Retrieved database query metrics ranked by descending value".to_string(),
        {
            json!({"query": input.query, "aggregator": aggregator, "order": "desc", "limit": input.limit.unwrap_or(100)})
        }
    )
}
