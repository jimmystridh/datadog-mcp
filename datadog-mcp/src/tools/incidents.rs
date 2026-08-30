//! Incident management tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use datadog_api::OffsetPage;
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

pub async fn get_incidents(
    ctx: ToolContext,
    page_size: Option<i32>,
    page_offset: Option<i64>,
) -> anyhow::Result<ToolOutput> {
    info!("Getting incidents");

    let page = OffsetPage::new(page_size.unwrap_or(25), page_offset)?;

    let api = ctx.incidents_api();
    let result = api.list_incidents(page).await;

    tool_response_with_fields!(
        result,
        no_cache,
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
