//! Event tools

use crate::response::tool_error;
use crate::sanitize::{
    sanitize_message, sanitize_name, sanitize_optional, sanitize_tags, MAX_NAME_LENGTH,
};
use crate::state::ToolContext;
use datadog_api::models::*;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_events(
    ctx: ToolContext,
    start: i64,
    end: i64,
    priority: Option<String>,
    sources: Option<String>,
) -> anyhow::Result<Value> {
    info!("Getting events from {} to {}", start, end);

    let api = ctx.events_api();
    let result = api
        .list_events(start, end, priority.as_deref(), sources.as_deref())
        .await;

    tool_response_with_fields!(
        result,
        "events",
        ctx,
        data,
        {
            let event_count = data.events.as_ref().map(|e| e.len()).unwrap_or(0);
            format!("Retrieved {} events", event_count)
        },
        {
            let event_count = data.events.as_ref().map(|e| e.len()).unwrap_or(0);
            json!({
                "event_count": event_count,
                "time_range": format!("{} to {}", start, end),
                "priority_filter": priority,
                "sources_filter": sources,
            })
        }
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn create_event(
    ctx: ToolContext,
    title: String,
    text: String,
    tags: Option<Vec<String>>,
    alert_type: Option<String>,
    priority: Option<String>,
    host: Option<String>,
    source_type_name: Option<String>,
    aggregation_key: Option<String>,
    date_happened: Option<i64>,
    device_name: Option<String>,
    related_event_id: Option<i64>,
) -> anyhow::Result<Value> {
    let title = sanitize_name(&title);
    let text = sanitize_message(&text);

    if title.is_empty() {
        return Ok(tool_error("create_event", "Empty title is not allowed"));
    }
    if text.is_empty() {
        return Ok(tool_error("create_event", "Empty text is not allowed"));
    }

    let tags = tags.map(sanitize_tags);
    let alert_type = sanitize_optional(alert_type, MAX_NAME_LENGTH);
    let priority = sanitize_optional(priority, MAX_NAME_LENGTH);
    let host = sanitize_optional(host, MAX_NAME_LENGTH);
    let source_type_name = sanitize_optional(source_type_name, MAX_NAME_LENGTH);
    let aggregation_key = sanitize_optional(aggregation_key, MAX_NAME_LENGTH);
    let device_name = sanitize_optional(device_name, MAX_NAME_LENGTH);

    info!("Creating event: {}", title);

    let request = EventCreateRequest {
        title: title.clone(),
        text: text.clone(),
        aggregation_key,
        alert_type,
        date_happened,
        device_name,
        host,
        priority,
        related_event_id,
        source_type_name,
        tags,
    };

    let api = ctx.events_api();
    let result = api.post_event(&request).await;

    tool_response_with_fields!(
        result,
        "event_created",
        ctx,
        data,
        {
            let event_id = data.event.as_ref().and_then(|e| e.id);
            match event_id {
                Some(id) => format!("Created event '{}' (ID: {})", title, id),
                None => format!("Created event '{}'", title),
            }
        },
        {
            let event_id = data.event.as_ref().and_then(|e| e.id);
            let event_id_str = data.event.as_ref().and_then(|e| e.id_str.clone());
            let event_title = data.event.as_ref().and_then(|e| e.title.clone());
            json!({
                "event_id": event_id,
                "event_id_str": event_id_str,
                "title": event_title.or_else(|| Some(title.clone())),
                "status": data.status.clone(),
            })
        }
    )
}

pub async fn get_event(ctx: ToolContext, event_id: i64) -> anyhow::Result<Value> {
    info!("Getting event: {}", event_id);

    let api = ctx.events_api();
    let result = api.get_event(event_id).await;

    tool_response_with_fields!(
        result,
        "event",
        ctx,
        data,
        {
            let title = data
                .event
                .as_ref()
                .and_then(|e| e.title.as_deref())
                .unwrap_or("Unknown");
            format!("Retrieved event: {} (ID: {})", title, event_id)
        },
        {
            let event_id_str = data.event.as_ref().and_then(|e| e.id_str.clone());
            let title = data.event.as_ref().and_then(|e| e.title.clone());
            json!({
                "event_id": event_id,
                "event_id_str": event_id_str,
                "title": title,
                "status": data.status.clone(),
            })
        }
    )
}
