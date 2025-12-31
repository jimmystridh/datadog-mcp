//! Dashboard tools

use crate::ids::DashboardId;
use crate::response::{simple_success_with_fields, tool_error};
use crate::sanitize::{sanitize_name, sanitize_optional, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH};
use crate::state::ToolContext;
use crate::input_validation::{validate_dashboard_layout, validate_dashboard_title};
use datadog_api::models::*;
use serde_json::{json, Value};
use tracing::info;

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

pub async fn get_dashboard(ctx: ToolContext, dashboard_id: DashboardId) -> anyhow::Result<Value> {
    info!("Getting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    let result = api.get_dashboard(&dashboard_id.0).await;

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
    let title = sanitize_name(&title);
    let description = sanitize_optional(description, MAX_MESSAGE_LENGTH);

    // Validate inputs
    if let Err(e) = validate_dashboard_title(&title) {
        return Ok(tool_error("create_dashboard", e));
    }
    if let Err(e) = validate_dashboard_layout(&layout_type) {
        return Ok(tool_error("create_dashboard", e));
    }

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
    dashboard_id: DashboardId,
    title: Option<String>,
    widgets: Option<Vec<Value>>,
) -> anyhow::Result<Value> {
    let title = sanitize_optional(title, MAX_NAME_LENGTH);

    info!("Updating dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();

    // Get existing dashboard first
    let existing = api.get_dashboard(&dashboard_id.0).await?;

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
        .update_dashboard(&dashboard_id.0, &updated_dashboard)
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

pub async fn delete_dashboard(ctx: ToolContext, dashboard_id: DashboardId) -> anyhow::Result<Value> {
    info!("Deleting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    let result = api.delete_dashboard(&dashboard_id.0).await;

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
