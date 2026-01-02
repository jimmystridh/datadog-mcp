//! Team tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_teams(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting teams");

    let api = ctx.teams_api();
    let result = api.list_teams().await;

    tool_response_with_fields!(
        result,
        "teams",
        ctx,
        data,
        {
            let teams = data.data.as_ref().map(|t| t.len()).unwrap_or(0);
            format!("Retrieved {} teams", teams)
        },
        {
            let teams = data.data.as_ref().map(|t| t.len()).unwrap_or(0);
            json!({
                "total_teams": teams,
            })
        }
    )
}
