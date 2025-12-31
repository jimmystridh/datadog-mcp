//! Incident management tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

pub async fn get_incidents(ctx: ToolContext, page_size: Option<i32>) -> anyhow::Result<Value> {
    info!("Getting incidents");

    let api = ctx.incidents_api();
    let result = api.list_all_incidents(page_size.unwrap_or(25)).await;

    tool_response_with_fields!(
        result,
        "incidents",
        ctx,
        data,
        format!("Retrieved {} incidents", data.len()),
        {
            let mut states: HashMap<String, usize> = HashMap::new();
            for incident in &data {
                if let Some(attrs) = &incident.attributes {
                    if let Some(state) = &attrs.state {
                        *states.entry(state.clone()).or_insert(0) += 1;
                    }
                }
            }

            json!({
                "total_incidents": data.len(),
                "incident_states": states,
                "active_incidents": states.get("active").copied().unwrap_or(0),
            })
        }
    )
}
