use crate::cache::store_data;
use crate::response::tool_error;
use crate::state::ToolContext;
use datadog_api::{apis::*, models::*};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info};

// ============================================================================
// METRICS & MONITORING TOOLS
// ============================================================================

pub async fn validate_api_key(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Validating API credentials");

    let api = MonitorsApi::new((*ctx.client).clone());
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

    let api = MetricsApi::new((*ctx.client).clone());
    let result = api
        .query_metrics(from_timestamp, to_timestamp, &query)
        .await;

    match result {
        Ok(data) => {
            let series = data.series.clone().unwrap_or_default();
            let series_count = series.len();
            let total_points: usize = series
                .iter()
                .map(|s| s.pointlist.as_ref().map(|p| p.len()).unwrap_or(0))
                .sum();

            let filepath = store_data(&data, "metrics", ctx.output_format).await?;
            info!(
                "Retrieved {} metric series with {} data points",
                series_count, total_points
            );

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} metric series with {} data points", series_count, total_points),
                "series_count": series_count,
                "data_points": total_points,
                "query": query,
                "time_range": format!("{} to {}", from_timestamp, to_timestamp),
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to get metrics", e)),
    }
}

pub async fn search_metrics(ctx: ToolContext, query: String) -> anyhow::Result<Value> {
    info!("Searching metrics: {}", query);

    let api = MetricsApi::new((*ctx.client).clone());
    let result = api.list_metrics(&query).await;

    match result {
        Ok(data) => {
            let metrics = data.metrics.clone().unwrap_or_default();
            let sample_metrics: Vec<_> = metrics.iter().take(10).cloned().collect();
            let metric_count = metrics.len();

            let filepath = store_data(&data, "metrics_search", ctx.output_format).await?;
            info!("Found {} metrics matching '{}'", metric_count, query);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Found {} metrics matching '{}'", metric_count, query),
                "metric_count": metric_count,
                "sample_metrics": sample_metrics,
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to search metrics", e)),
    }
}

pub async fn get_metric_metadata(
    ctx: ToolContext,
    metric_name: String,
) -> anyhow::Result<Value> {
    info!("Getting metadata for metric: {}", metric_name);

    let api = MetricsApi::new((*ctx.client).clone());
    let result = api.get_metric_metadata(&metric_name).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "metric_metadata", ctx.output_format).await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved metadata for metric: {}", metric_name),
                "metric_name": metric_name,
                "description": data.description.unwrap_or_else(|| "No description".to_string()),
                "unit": data.unit.unwrap_or_else(|| "No unit".to_string()),
                "type": data.metric_type.unwrap_or_else(|| "Unknown".to_string()),
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to get metric metadata", e)),
    }
}

pub async fn get_monitors(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting all monitors");

    let api = MonitorsApi::new((*ctx.client).clone());
    let result = api.list_monitors().await;

    match result {
        Ok(data) => {
            let mut states: HashMap<String, usize> = HashMap::new();

            for monitor in &data {
                if let Some(state) = &monitor.overall_state {
                    *states.entry(state.clone()).or_insert(0) += 1;
                }
            }

            let filepath = store_data(&data, "monitors", ctx.output_format).await?;
            info!("Retrieved {} monitors", data.len());

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} monitors", data.len()),
                "total_monitors": data.len(),
                "monitor_states": states,
                "alerting_count": states.get("Alert").unwrap_or(&0),
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to get monitors", e)),
    }
}

pub async fn get_monitor(ctx: ToolContext, monitor_id: i64) -> anyhow::Result<Value> {
    info!("Getting monitor: {}", monitor_id);

    let api = MonitorsApi::new((*ctx.client).clone());
    let result = api.get_monitor(monitor_id).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "monitor", ctx.output_format).await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Monitor: {} - Status: {}",
                    data.name.as_ref().unwrap_or(&"Unknown".to_string()),
                    data.overall_state.as_ref().unwrap_or(&"Unknown".to_string())
                ),
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": data.overall_state.map(|s| if s == "Alert" { "alerting" } else { "ok" }),
                "monitor_type": data.monitor_type,
            }))
        }
        Err(e) => Ok(tool_error(&format!("Failed to get monitor {}", monitor_id), e)),
    }
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

    let api = MonitorsApi::new((*ctx.client).clone());
    let result = api.create_monitor(&request).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "monitor_created", ctx.output_format).await?;
            info!("Created monitor: {} (ID: {:?})", name, data.id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Created monitor: {} (ID: {:?})", name, data.id),
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": "created",
            }))
        }
        Err(e) => Ok(tool_error("Failed to create monitor", e)),
    }
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

    let api = MonitorsApi::new((*ctx.client).clone());
    let result = api.update_monitor(monitor_id, &request).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "monitor_updated", ctx.output_format).await?;
            info!("Updated monitor: {:?} (ID: {:?})", data.name, data.id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Updated monitor: {:?} (ID: {:?})", data.name, data.id),
                "monitor_id": data.id,
                "monitor_name": data.name,
                "status": "updated",
            }))
        }
        Err(e) => Ok(tool_error("Failed to update monitor", e)),
    }
}

pub async fn delete_monitor(ctx: ToolContext, monitor_id: i64) -> anyhow::Result<Value> {
    info!("Deleting monitor: {}", monitor_id);

    let api = MonitorsApi::new((*ctx.client).clone());
    let result = api.delete_monitor(monitor_id).await;

    match result {
        Ok(_) => {
            info!("Successfully deleted monitor ID: {}", monitor_id);
            Ok(json!({
                "summary": format!("Successfully deleted monitor ID: {}", monitor_id),
                "monitor_id": monitor_id,
                "status": "deleted",
            }))
        }
        Err(e) => Ok(tool_error("Failed to delete monitor", e)),
    }
}

// ============================================================================
// DASHBOARD TOOLS
// ============================================================================

pub async fn get_dashboards(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting all dashboards");

    let api = DashboardsApi::new((*ctx.client).clone());
    let result = api.list_dashboards().await;

    match result {
        Ok(data) => {
            let dashboards = data.dashboards.clone().unwrap_or_default();
            let sample_dashboards: Vec<_> = dashboards
                .iter()
                .take(5)
                .map(|d| d.title.as_ref().unwrap_or(&"Untitled".to_string()).clone())
                .collect();
            let dashboard_count = dashboards.len();

            let filepath = store_data(&data, "dashboards", ctx.output_format).await?;
            info!("Retrieved {} dashboards", dashboard_count);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} dashboards", dashboard_count),
                "total_dashboards": dashboard_count,
                "sample_dashboards": sample_dashboards,
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to get dashboards", e)),
    }
}

pub async fn get_dashboard(ctx: ToolContext, dashboard_id: String) -> anyhow::Result<Value> {
    info!("Getting dashboard: {}", dashboard_id);

    let api = DashboardsApi::new((*ctx.client).clone());
    let result = api.get_dashboard(&dashboard_id).await;

    match result {
        Ok(data) => {
            let widgets = data.widgets.as_ref().map(|w| w.len()).unwrap_or(0);
            let filepath = store_data(&data, "dashboard", ctx.output_format).await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Dashboard: {:?} with {} widgets",
                    data.title.as_ref().unwrap_or(&"Untitled".to_string()),
                    widgets
                ),
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "widget_count": widgets,
                "layout_type": data.layout_type,
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error(&format!("Failed to get dashboard {}", dashboard_id), e)),
    }
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

    let api = DashboardsApi::new((*ctx.client).clone());
    let result = api.create_dashboard(&dashboard).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "dashboard_created", ctx.output_format).await?;
            info!("Created dashboard: {:?} (ID: {:?})", data.title, data.id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Created dashboard: {:?} (ID: {:?})", data.title, data.id),
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "status": "created",
            }))
        }
        Err(e) => Ok(tool_error("Failed to create dashboard", e)),
    }
}

pub async fn update_dashboard(
    ctx: ToolContext,
    dashboard_id: String,
    title: Option<String>,
    widgets: Option<Vec<Value>>,
) -> anyhow::Result<Value> {
    info!("Updating dashboard: {}", dashboard_id);

    let api = DashboardsApi::new((*ctx.client).clone());

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

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "dashboard_updated", ctx.output_format).await?;
            info!("Updated dashboard: {:?} (ID: {:?})", data.title, data.id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Updated dashboard: {:?} (ID: {:?})", data.title, data.id),
                "dashboard_id": data.id,
                "dashboard_title": data.title,
                "status": "updated",
            }))
        }
        Err(e) => Ok(tool_error("Failed to update dashboard", e)),
    }
}

pub async fn delete_dashboard(
    ctx: ToolContext,
    dashboard_id: String,
) -> anyhow::Result<Value> {
    info!("Deleting dashboard: {}", dashboard_id);

    let api = DashboardsApi::new((*ctx.client).clone());
    let result = api.delete_dashboard(&dashboard_id).await;

    match result {
        Ok(_) => {
            info!("Successfully deleted dashboard ID: {}", dashboard_id);
            Ok(json!({
                "summary": format!("Successfully deleted dashboard ID: {}", dashboard_id),
                "dashboard_id": dashboard_id,
                "status": "deleted",
            }))
        }
        Err(e) => Ok(tool_error("Failed to delete dashboard", e)),
    }
}

// Continue in next file due to size...
