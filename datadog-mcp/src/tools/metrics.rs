//! Metrics tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_metrics(
    ctx: ToolContext,
    query: String,
    from_timestamp: i64,
    to_timestamp: i64,
) -> anyhow::Result<Value> {
    info!("Querying metrics: {}", query);

    let api = ctx.metrics_api();
    let result = api
        .query_metrics(from_timestamp, to_timestamp, &query)
        .await;

    tool_response_with_fields!(
        result,
        "metrics",
        ctx,
        data,
        {
            let series_count = data.series.as_ref().map(|s| s.len()).unwrap_or(0);
            let total_points: usize = data
                .series
                .as_ref()
                .map(|s| {
                    s.iter()
                        .map(|series| series.pointlist.as_ref().map(|p| p.len()).unwrap_or(0))
                        .sum()
                })
                .unwrap_or(0);
            format!(
                "Retrieved {} metric series with {} data points",
                series_count, total_points
            )
        },
        {
            let series_count = data.series.as_ref().map(|s| s.len()).unwrap_or(0);
            let total_points: usize = data
                .series
                .as_ref()
                .map(|s| {
                    s.iter()
                        .map(|series| series.pointlist.as_ref().map(|p| p.len()).unwrap_or(0))
                        .sum()
                })
                .unwrap_or(0);

            json!({
                "series_count": series_count,
                "data_points": total_points,
                "query": query,
                "time_range": format!("{} to {}", from_timestamp, to_timestamp),
            })
        }
    )
}

pub async fn search_metrics(ctx: ToolContext, query: String) -> anyhow::Result<Value> {
    info!("Searching metrics: {}", query);

    let api = ctx.metrics_api();
    let result = api.list_metrics(&query).await;

    tool_response_with_fields!(
        result,
        "metrics_search",
        ctx,
        data,
        {
            let metric_count = data.metrics.as_ref().map(|m| m.len()).unwrap_or(0);
            format!("Found {} metrics matching '{}'", metric_count, query)
        },
        {
            let metrics = data.metrics.clone().unwrap_or_default();
            let sample_metrics: Vec<_> = metrics.iter().take(10).cloned().collect();
            json!({
                "metric_count": metrics.len(),
                "sample_metrics": sample_metrics,
            })
        }
    )
}

pub async fn get_metric_metadata(ctx: ToolContext, metric_name: String) -> anyhow::Result<Value> {
    info!("Getting metadata for metric: {}", metric_name);

    let api = ctx.metrics_api();
    let result = api.get_metric_metadata(&metric_name).await;

    tool_response_with_fields!(
        result,
        "metric_metadata",
        ctx,
        data,
        format!("Retrieved metadata for metric: {}", metric_name),
        {
            json!({
                "metric_name": metric_name,
                "description": data.description.clone().unwrap_or_else(|| "No description".to_string()),
                "unit": data.unit.clone().unwrap_or_else(|| "No unit".to_string()),
                "type": data.metric_type.clone().unwrap_or_else(|| "Unknown".to_string()),
            })
        }
    )
}
