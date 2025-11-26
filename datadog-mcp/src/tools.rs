use crate::response::{simple_success_with_fields, tool_error};
use crate::state::ToolContext;
use crate::tool_response_with_fields;
use datadog_api::models::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info};

// ============================================================================
// METRICS & MONITORING TOOLS
// ============================================================================

pub async fn validate_api_key(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Validating API credentials");

    let api = ctx.monitors_api();
    let result = api.list_monitors_with_page_size(1).await;

    match result {
        Ok(_) => {
            info!("API credentials validated successfully");
            Ok(json!({
                "valid": true,
                "summary": "API credentials are valid and working",
                "site": ctx.client.config().site,
                "test_successful": true,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("API validation failed: {}", e);
            Ok(json!({
                "valid": false,
                "error": format!("API validation failed: {}", e),
                "site": ctx.client.config().site,
                "status": "error",
            }))
        }
    }
}

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

pub async fn get_monitors(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting all monitors");

    let api = ctx.monitors_api();
    let result = api.list_monitors().await;

    tool_response_with_fields!(
        result,
        "monitors",
        ctx,
        data,
        format!("Retrieved {} monitors", data.len()),
        {
            let mut states: HashMap<String, usize> = HashMap::new();
            for monitor in &data {
                if let Some(state) = &monitor.overall_state {
                    *states.entry(state.clone()).or_insert(0) += 1;
                }
            }
            json!({
                "total_monitors": data.len(),
                "monitor_states": states,
                "alerting_count": data.iter().filter(|m| m.overall_state.as_deref() == Some("Alert")).count(),
            })
        }
    )
}

pub async fn get_monitor(ctx: ToolContext, monitor_id: i64) -> anyhow::Result<Value> {
    info!("Getting monitor: {}", monitor_id);

    let api = ctx.monitors_api();
    let result = api.get_monitor(monitor_id).await;

    tool_response_with_fields!(
        result,
        "monitor",
        ctx,
        data,
        {
            format!(
                "Monitor: {} - Status: {}",
                data.name.as_deref().unwrap_or("Unknown"),
                data.overall_state.as_deref().unwrap_or("Unknown")
            )
        },
        {
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": data.overall_state.clone().map(|s| if s == "Alert" { "alerting" } else { "ok" }),
                "monitor_type": data.monitor_type,
            })
        }
    )
}

pub async fn create_monitor(
    ctx: ToolContext,
    name: String,
    monitor_type: String,
    query: String,
    message: Option<String>,
    options: Option<Value>,
) -> anyhow::Result<Value> {
    info!("Creating monitor: {}", name);

    let monitor_options = options.and_then(|v| serde_json::from_value(v).ok());

    let request = MonitorCreateRequest {
        name: name.clone(),
        monitor_type: monitor_type.clone(),
        query: query.clone(),
        message,
        tags: None,
        options: monitor_options,
    };

    let api = ctx.monitors_api();
    let result = api.create_monitor(&request).await;

    tool_response_with_fields!(
        result,
        "monitor_created",
        ctx,
        data,
        format!("Created monitor: {} (ID: {:?})", name, data.id),
        {
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": "created",
            })
        }
    )
}

pub async fn update_monitor(
    ctx: ToolContext,
    monitor_id: i64,
    name: Option<String>,
    query: Option<String>,
    message: Option<String>,
    options: Option<Value>,
) -> anyhow::Result<Value> {
    info!("Updating monitor: {}", monitor_id);

    let monitor_options = options.and_then(|v| serde_json::from_value(v).ok());

    let request = MonitorUpdateRequest {
        name,
        query,
        message,
        tags: None,
        options: monitor_options,
    };

    let api = ctx.monitors_api();
    let result = api.update_monitor(monitor_id, &request).await;

    tool_response_with_fields!(
        result,
        "monitor_updated",
        ctx,
        data,
        format!("Updated monitor: {:?} (ID: {:?})", data.name, data.id),
        {
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": "updated",
            })
        }
    )
}

pub async fn delete_monitor(ctx: ToolContext, monitor_id: i64) -> anyhow::Result<Value> {
    info!("Deleting monitor: {}", monitor_id);

    let api = ctx.monitors_api();
    let result = api.delete_monitor(monitor_id).await;

    match result {
        Ok(_) => {
            info!("Successfully deleted monitor ID: {}", monitor_id);
            Ok(simple_success_with_fields(
                format!("Successfully deleted monitor ID: {}", monitor_id),
                json!({
                    "monitor_id": monitor_id,
                    "status": "deleted",
                }),
            ))
        }
        Err(e) => Ok(tool_error("Failed to delete monitor", e)),
    }
}

// ============================================================================
// DASHBOARD TOOLS
// ============================================================================

pub async fn get_dashboards(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting all dashboards");

    let api = ctx.dashboards_api();
    let result = api.list_dashboards().await;

    tool_response_with_fields!(
        result,
        "dashboards",
        ctx,
        data,
        {
            let dashboard_count = data.dashboards.as_ref().map(|d| d.len()).unwrap_or(0);
            format!("Retrieved {} dashboards", dashboard_count)
        },
        {
            let dashboards = data.dashboards.clone().unwrap_or_default();
            let sample_dashboards: Vec<_> = dashboards
                .iter()
                .take(5)
                .map(|d| d.title.as_ref().unwrap_or(&"Untitled".to_string()).clone())
                .collect();
            json!({
                "total_dashboards": dashboards.len(),
                "sample_dashboards": sample_dashboards,
            })
        }
    )
}

pub async fn get_dashboard(ctx: ToolContext, dashboard_id: String) -> anyhow::Result<Value> {
    info!("Getting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    let result = api.get_dashboard(&dashboard_id).await;

    tool_response_with_fields!(
        result,
        "dashboard",
        ctx,
        data,
        {
            let widgets = data.widgets.as_ref().map(|w| w.len()).unwrap_or(0);
            format!(
                "Dashboard: {:?} with {} widgets",
                data.title.as_ref().unwrap_or(&"Untitled".to_string()),
                widgets
            )
        },
        {
            let widgets = data.widgets.as_ref().map(|w| w.len()).unwrap_or(0);
            json!({
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "widget_count": widgets,
                "layout_type": data.layout_type,
            })
        }
    )
}

pub async fn create_dashboard(
    ctx: ToolContext,
    title: String,
    layout_type: String,
    widgets: Vec<Value>,
    description: Option<String>,
) -> anyhow::Result<Value> {
    info!("Creating dashboard: {}", title);

    let dashboard = Dashboard {
        id: None,
        title: Some(title.clone()),
        description,
        widgets: Some(widgets),
        layout_type: Some(layout_type),
        is_read_only: None,
        notify_list: None,
        template_variables: None,
    };

    let api = ctx.dashboards_api();
    let result = api.create_dashboard(&dashboard).await;

    tool_response_with_fields!(
        result,
        "dashboard_created",
        ctx,
        data,
        format!("Created dashboard: {:?} (ID: {:?})", data.title, data.id),
        {
            json!({
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "status": "created",
            })
        }
    )
}

pub async fn update_dashboard(
    ctx: ToolContext,
    dashboard_id: String,
    title: Option<String>,
    widgets: Option<Vec<Value>>,
) -> anyhow::Result<Value> {
    info!("Updating dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();

    // Get existing dashboard first
    let existing = api.get_dashboard(&dashboard_id).await?;

    let updated_dashboard = Dashboard {
        id: existing.id,
        title: title.or(existing.title),
        description: existing.description,
        widgets: widgets.or(existing.widgets),
        layout_type: existing.layout_type,
        is_read_only: existing.is_read_only,
        notify_list: existing.notify_list,
        template_variables: existing.template_variables,
    };

    let result = api
        .update_dashboard(&dashboard_id, &updated_dashboard)
        .await;

    tool_response_with_fields!(
        result,
        "dashboard_updated",
        ctx,
        data,
        format!("Updated dashboard: {:?} (ID: {:?})", data.title, data.id),
        {
            json!({
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "status": "updated",
            })
        }
    )
}

pub async fn delete_dashboard(ctx: ToolContext, dashboard_id: String) -> anyhow::Result<Value> {
    info!("Deleting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    let result = api.delete_dashboard(&dashboard_id).await;

    match result {
        Ok(_) => {
            info!("Successfully deleted dashboard ID: {}", dashboard_id);
            Ok(simple_success_with_fields(
                format!("Successfully deleted dashboard ID: {}", dashboard_id),
                json!({
                    "dashboard_id": dashboard_id,
                    "status": "deleted",
                }),
            ))
        }
        Err(e) => Ok(tool_error("Failed to delete dashboard", e)),
    }
}

// Continue in next file due to size...
