//! Monitor tools

use crate::response::{simple_success_with_fields, tool_error};
use crate::sanitize::{sanitize_message, sanitize_name, sanitize_optional, sanitize_query, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH, MAX_QUERY_LENGTH};
use crate::state::ToolContext;
use crate::tool_inputs::{MonitorId, MonitorOptions};
use datadog_api::models::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

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

pub async fn get_monitor(ctx: ToolContext, monitor_id: MonitorId) -> anyhow::Result<Value> {
    info!("Getting monitor: {}", monitor_id.0);

    let api = ctx.monitors_api();
    let result = api.get_monitor(monitor_id.0).await;

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
    options: Option<MonitorOptions>,
) -> anyhow::Result<Value> {
    let name = sanitize_name(&name);
    let query = sanitize_query(&query);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);

    info!("Creating monitor: {}", name);

    let request = MonitorCreateRequest {
        name: name.clone(),
        monitor_type: monitor_type.clone(),
        query: query.clone(),
        message,
        tags: None,
        options: options.map(|opt| opt.into()),
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
    monitor_id: MonitorId,
    name: Option<String>,
    query: Option<String>,
    message: Option<String>,
    options: Option<MonitorOptions>,
) -> anyhow::Result<Value> {
    let name = sanitize_optional(name, MAX_NAME_LENGTH);
    let query = sanitize_optional(query, MAX_QUERY_LENGTH);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);

    info!("Updating monitor: {}", monitor_id.0);

    let request = MonitorUpdateRequest {
        name,
        query,
        message,
        tags: None,
        options: options.map(|opt| opt.into()),
    };

    let api = ctx.monitors_api();
    let result = api.update_monitor(monitor_id.0, &request).await;

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

pub async fn delete_monitor(ctx: ToolContext, monitor_id: MonitorId) -> anyhow::Result<Value> {
    info!("Deleting monitor: {}", monitor_id.0);

    let api = ctx.monitors_api();
    let result = api.delete_monitor(monitor_id.0).await;

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
