//! Metrics tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use datadog_api::{models::MetricsListResponse, TimestampSecs};
use serde_json::json;
use tracing::info;

pub async fn get_metrics(
    ctx: ToolContext,
    query: String,
    from_timestamp: i64,
    to_timestamp: i64,
) -> anyhow::Result<ToolOutput> {
    info!(query_length = query.len(), "Querying metrics");

    let api = ctx.metrics_api();
    let result = api
        .query_metrics(from_timestamp, to_timestamp, &query)
        .await;

    tool_response_with_fields!(
        result,
        cache("metrics"),
        data,
        {
            let series_count = data
                .data
                .as_ref()
                .map_or(0, |value| value.attributes.series.len());
            let total_points: usize = data.data.as_ref().map_or(0, |value| {
                value.attributes.values.iter().map(Vec::len).sum()
            });
            format!(
                "Retrieved {} metric series with {} data points",
                series_count, total_points
            )
        },
        {
            let series_count = data
                .data
                .as_ref()
                .map_or(0, |value| value.attributes.series.len());
            let total_points: usize = data.data.as_ref().map_or(0, |value| {
                value.attributes.values.iter().map(Vec::len).sum()
            });

            json!({
                "series_count": series_count,
                "data_points": total_points,
                "query": query,
                "time_range": format!("{} to {}", from_timestamp, to_timestamp),
            })
        }
    )
}

pub async fn search_metrics(
    ctx: ToolContext,
    query: String,
    from_timestamp: Option<i64>,
) -> anyhow::Result<ToolOutput> {
    info!("Searching metrics: {}", query);

    let from_timestamp = from_timestamp.unwrap_or_else(|| TimestampSecs::now().0 - 86_400);
    let api = ctx.metrics_api();
    let result = api.list_active_metrics(from_timestamp).await.map(|data| {
        let query = query.to_lowercase();
        let metrics = data
            .metrics
            .unwrap_or_default()
            .into_iter()
            .filter(|metric| metric.to_lowercase().contains(&query))
            .collect();
        MetricsListResponse {
            metrics: Some(metrics),
        }
    });

    tool_response_with_fields!(
        result,
        cache("metrics_search"),
        data,
        {
            let metric_count = data.metrics.as_ref().map(|m| m.len()).unwrap_or(0);
            format!("Found {} metrics matching '{}'", metric_count, query)
        },
        {
            let metrics = data.metrics.clone().unwrap_or_default();
            let sample_metrics: Vec<_> = metrics.iter().take(10).cloned().collect();
            json!({
                "from_timestamp": from_timestamp,
                "metric_count": metrics.len(),
                "sample_metrics": sample_metrics,
            })
        }
    )
}

pub async fn get_metric_metadata(
    ctx: ToolContext,
    metric_name: String,
) -> anyhow::Result<ToolOutput> {
    info!("Getting metadata for metric: {}", metric_name);

    let api = ctx.metrics_api();
    let result = api.get_metric_metadata(&metric_name).await;

    tool_response_with_fields!(
        result,
        cache("metric_metadata"),
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
