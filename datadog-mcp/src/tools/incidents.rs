//! Incident management tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

pub async fn get_incidents(
    ctx: ToolContext,
    page_size: Option<i32>,
    page_offset: Option<i64>,
) -> anyhow::Result<Value> {
    info!("Getting incidents");

    let page_size = page_size.unwrap_or(25);
    if !(1..=100).contains(&page_size) {
        anyhow::bail!("Incident page size must be between 1 and 100");
    }
    if page_offset.is_some_and(|offset| offset < 0) {
        anyhow::bail!("Incident page offset cannot be negative");
    }

    let api = ctx.incidents_api();
    let result = api.list_incidents(Some(page_size), page_offset).await;

    tool_response_with_fields!(
        result,
        "incidents",
        ctx,
        data,
        format!(
            "Retrieved {} incidents",
            data.data.as_ref().map_or(0, Vec::len)
        ),
        {
            let mut states: HashMap<String, usize> = HashMap::new();
            for incident in data.data.as_deref().unwrap_or_default() {
                if let Some(attrs) = &incident.attributes {
                    if let Some(state) = &attrs.state {
                        *states.entry(state.clone()).or_insert(0) += 1;
                    }
                }
            }

            json!({
                "total_incidents": data.data.as_ref().map_or(0, Vec::len),
                "incident_states": states,
                "active_incidents": states.get("active").copied().unwrap_or(0),
                "next_offset": data.meta.as_ref()
                    .and_then(|meta| meta.pagination.as_ref())
                    .and_then(|pagination| pagination.next_offset),
            })
        }
    )
}
