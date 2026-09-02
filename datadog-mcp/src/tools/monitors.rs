//! Monitor tools

use crate::input_validation::{
    validate_monitor_name, validate_monitor_query, validate_monitor_type,
};
use crate::response::{simple_success_with_fields, ToolOutput};
use crate::sanitize::{
    sanitize_name, sanitize_optional, sanitize_query, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH,
    MAX_QUERY_LENGTH,
};
use crate::state::ToolContext;
use crate::tool_inputs::{MonitorGroupStateFilter, MonitorId, MonitorOptions};
use datadog_api::apis::GetMonitorOptions;
use datadog_api::models::*;
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

pub async fn get_monitors(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Getting all monitors");

    let api = ctx.monitors_api();
    let result = api.list_monitors().await;

    tool_response_with_fields!(
        result,
        cache("monitors"),
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

pub async fn search_monitors(
    ctx: ToolContext,
    query: String,
    page: Option<i64>,
    per_page: Option<i64>,
    sort: Option<String>,
) -> anyhow::Result<ToolOutput> {
    let query = sanitize_query(&query);
    let sort = sanitize_optional(sort, MAX_NAME_LENGTH);

    if let Err(e) = validate_monitor_query(&query) {
        return Err(e.into());
    }

    info!("Searching monitors: {}", query);

    let api = ctx.monitors_api();
    let result = api
        .search_monitors(&query, page, per_page, sort.as_deref())
        .await;

    tool_response_with_fields!(
        result,
        cache("monitors_search"),
        data,
        {
            let returned = data.monitors.as_ref().map(|m| m.len()).unwrap_or(0);
            let total = data
                .metadata
                .as_ref()
                .and_then(|m| m.total_count)
                .unwrap_or(0);
            format!("Found {} monitor(s) ({} returned)", total, returned)
        },
        {
            let returned = data.monitors.as_ref().map(|m| m.len()).unwrap_or(0);
            let total = data
                .metadata
                .as_ref()
                .and_then(|m| m.total_count)
                .unwrap_or(0);
            json!({
                "query": query.clone(),
                "returned": returned,
                "total": total,
                "page": data.metadata.as_ref().and_then(|m| m.page),
                "per_page": data.metadata.as_ref().and_then(|m| m.per_page),
                "page_count": data.metadata.as_ref().and_then(|m| m.page_count),
            })
        }
    )
}

pub async fn get_monitor(
    ctx: ToolContext,
    monitor_id: MonitorId,
    group_states: Option<Vec<MonitorGroupStateFilter>>,
    with_downtimes: Option<bool>,
) -> anyhow::Result<ToolOutput> {
    info!("Getting monitor: {}", monitor_id.0);

    let group_states = match group_states {
        None => Some("all".to_string()),
        Some(states) if states.is_empty() => None,
        Some(states) => Some(
            states
                .into_iter()
                .map(MonitorGroupStateFilter::as_query_value)
                .collect::<Vec<_>>()
                .join(","),
        ),
    };
    let with_downtimes = with_downtimes.unwrap_or(true);
    let options = GetMonitorOptions {
        group_states: group_states.as_deref(),
        with_downtimes: Some(with_downtimes),
    };
    let api = ctx.monitors_api();
    let result = api.get_monitor_with_options(monitor_id.0, &options).await;

    tool_response_with_fields!(
        result,
        cache("monitor"),
        data,
        {
            format!(
                "Monitor: {} - Status: {}",
                data.name.as_deref().unwrap_or("Unknown"),
                data.overall_state.as_deref().unwrap_or("Unknown")
            )
        },
        {
            let groups = data.state.as_ref().and_then(|state| state.groups.as_ref());
            let group_count = groups.map_or(0, HashMap::len);
            let mut group_status_counts = HashMap::new();
            if let Some(groups) = groups {
                for group in groups.values() {
                    let status = group.status.as_deref().unwrap_or("Unknown");
                    *group_status_counts.entry(status).or_insert(0) += 1;
                }
            }
            let alerting_group_count = groups.map_or(0, |groups| {
                groups
                    .values()
                    .filter(|group| group.status.as_deref() == Some("Alert"))
                    .count()
            });
            let warning_group_count = groups.map_or(0, |groups| {
                groups
                    .values()
                    .filter(|group| group.status.as_deref() == Some("Warn"))
                    .count()
            });
            let no_data_group_count = groups.map_or(0, |groups| {
                groups
                    .values()
                    .filter(|group| group.status.as_deref() == Some("No Data"))
                    .count()
            });
            let status = data.overall_state.as_deref().map(|state| match state {
                "Alert" => "alerting",
                "Warn" => "warning",
                "No Data" => "no_data",
                "OK" => "ok",
                "Ignored" => "ignored",
                "Skipped" => "skipped",
                _ => "unknown",
            });
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "monitor_status": status,
                "monitor_type": data.monitor_type,
                "group_count": group_count,
                "group_status_counts": group_status_counts,
                "alerting_group_count": alerting_group_count,
                "warning_group_count": warning_group_count,
                "no_data_group_count": no_data_group_count,
                "matching_downtime_count": data.matching_downtimes.as_ref().map_or(0, Vec::len),
                "requested_group_states": group_states,
                "requested_with_downtimes": with_downtimes,
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
    tags: Option<Vec<String>>,
    options: Option<MonitorOptions>,
) -> anyhow::Result<ToolOutput> {
    let name = sanitize_name(&name);
    let query = sanitize_query(&query);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);

    // Validate inputs
    if let Err(e) = validate_monitor_name(&name) {
        return Err(e.into());
    }
    if let Err(e) = validate_monitor_type(&monitor_type) {
        return Err(e.into());
    }
    if let Err(e) = validate_monitor_query(&query) {
        return Err(e.into());
    }

    info!("Creating monitor: {}", name);

    let request = MonitorCreateRequest {
        name: name.clone(),
        monitor_type: monitor_type.clone(),
        query: query.clone(),
        message,
        tags,
        options: options.map(|opt| opt.into()),
    };

    let api = ctx.monitors_api();
    let result = api.create_monitor(&request).await;

    tool_response_with_fields!(
        result,
        cache("monitor_created"),
        data,
        format!("Created monitor: {} (ID: {:?})", name, data.id),
        {
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "operation_status": "created",
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
    tags: Option<Vec<String>>,
    options: Option<MonitorOptions>,
) -> anyhow::Result<ToolOutput> {
    let name = sanitize_optional(name, MAX_NAME_LENGTH);
    let query = sanitize_optional(query, MAX_QUERY_LENGTH);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);

    info!("Updating monitor: {}", monitor_id.0);

    let request = MonitorUpdateRequest {
        name,
        query,
        message,
        tags,
        options: options.map(|opt| opt.into()),
    };

    let api = ctx.monitors_api();
    let result = api.update_monitor(monitor_id.0, &request).await;

    tool_response_with_fields!(
        result,
        cache("monitor_updated"),
        data,
        format!("Updated monitor: {:?} (ID: {:?})", data.name, data.id),
        {
            json!({
                "monitor_id": data.id,
                "monitor_name": data.name,
                "operation_status": "updated",
            })
        }
    )
}

pub async fn delete_monitor(ctx: ToolContext, monitor_id: MonitorId) -> anyhow::Result<ToolOutput> {
    info!("Deleting monitor: {}", monitor_id.0);

    let api = ctx.monitors_api();
    api.delete_monitor(monitor_id.0).await?;
    info!("Successfully deleted monitor ID: {}", monitor_id);
    simple_success_with_fields(
        format!("Successfully deleted monitor ID: {}", monitor_id),
        json!({
            "monitor_id": monitor_id,
            "operation_status": "deleted",
        }),
    )
}
