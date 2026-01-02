//! Service Level Objectives tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_slos(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting Service Level Objectives");

    let api = ctx.slos_api();
    let result = api.list_slos().await;

    tool_response_with_fields!(
        result,
        "slos",
        ctx,
        data,
        {
            let slos = data.data.as_ref().map(|s| s.len()).unwrap_or(0);
            format!("Retrieved {} SLOs", slos)
        },
        {
            let slos = data.data.as_ref().map(|s| s.len()).unwrap_or(0);
            json!({
                "total_slos": slos,
            })
        }
    )
}
