//! Dashboard tools

use crate::ids::DashboardId;
use crate::input_validation::{validate_dashboard_layout, validate_dashboard_title};
use crate::response::{simple_success_with_fields, ToolOutput};
use crate::sanitize::{sanitize_name, sanitize_optional, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH};
use crate::state::ToolContext;
use datadog_api::{DashboardDocument, DashboardPatch};
use serde_json::{json, Value};
use tracing::info;

pub async fn get_dashboards(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Getting all dashboards");

    let api = ctx.dashboards_api();
    let result = api.list_dashboards().await;

    tool_response_with_fields!(
        result,
        cache("dashboards"),
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

pub async fn get_dashboard(
    ctx: ToolContext,
    dashboard_id: DashboardId,
) -> anyhow::Result<ToolOutput> {
    info!("Getting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    let result = api.get_dashboard(&dashboard_id.0).await;

    tool_response_with_fields!(
        result,
        cache("dashboard"),
        data,
        {
            format!(
                "Dashboard: {} with {} widgets",
                data.title().unwrap_or("Untitled"),
                data.widget_count()
            )
        },
        {
            json!({
                "dashboard_id": data.id(),
                "dashboard_title": data.title(),
                "widget_count": data.widget_count(),
                "layout_type": data.layout_type(),
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
) -> anyhow::Result<ToolOutput> {
    let title = sanitize_name(&title);
    let description = sanitize_optional(description, MAX_MESSAGE_LENGTH);

    // Validate inputs
    if let Err(e) = validate_dashboard_title(&title) {
        return Err(e.into());
    }
    if let Err(e) = validate_dashboard_layout(&layout_type) {
        return Err(e.into());
    }

    info!("Creating dashboard: {}", title);

    let dashboard = DashboardDocument::new(title, layout_type, widgets, description);

    let api = ctx.dashboards_api();
    let result = api.create_dashboard(&dashboard).await;

    tool_response_with_fields!(
        result,
        cache("dashboard_created"),
        data,
        format!("Created dashboard: {}", data.title().unwrap_or("Untitled")),
        {
            json!({
                "dashboard_id": data.id(),
                "dashboard_title": data.title(),
                "operation_status": "created",
            })
        }
    )
}

pub async fn update_dashboard(
    ctx: ToolContext,
    dashboard_id: DashboardId,
    title: Option<String>,
    widgets: Option<Vec<Value>>,
) -> anyhow::Result<ToolOutput> {
    let title = sanitize_optional(title, MAX_NAME_LENGTH);

    info!("Updating dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();

    // Get existing dashboard first
    let existing = api.get_dashboard(&dashboard_id.0).await?;

    let mut updated_dashboard = existing;
    updated_dashboard.apply_patch(DashboardPatch { title, widgets });
    let updated_dashboard = updated_dashboard.into_update_payload();

    let result = api
        .update_dashboard(&dashboard_id.0, &updated_dashboard)
        .await;

    tool_response_with_fields!(
        result,
        cache("dashboard_updated"),
        data,
        format!("Updated dashboard: {}", data.title().unwrap_or("Untitled")),
        {
            json!({
                "dashboard_id": data.id(),
                "dashboard_title": data.title(),
                "operation_status": "updated",
            })
        }
    )
}

pub async fn delete_dashboard(
    ctx: ToolContext,
    dashboard_id: DashboardId,
) -> anyhow::Result<ToolOutput> {
    info!("Deleting dashboard: {}", dashboard_id);

    let api = ctx.dashboards_api();
    api.delete_dashboard(&dashboard_id.0).await?;
    info!("Successfully deleted dashboard ID: {}", dashboard_id);
    simple_success_with_fields(
        format!("Successfully deleted dashboard ID: {}", dashboard_id),
        json!({
            "dashboard_id": dashboard_id,
            "operation_status": "deleted",
        }),
    )
}
