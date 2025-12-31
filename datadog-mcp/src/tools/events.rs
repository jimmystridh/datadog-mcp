//! Event tools

use crate::state::ToolContext;
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
