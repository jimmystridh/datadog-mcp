//! Team tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_teams(
    ctx: ToolContext,
    page_number: Option<i64>,
    page_size: Option<i64>,
) -> anyhow::Result<Value> {
    info!("Getting teams");

    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(100);
    if page_number < 1 || !(1..=100).contains(&page_size) {
        anyhow::bail!("Team page number must be positive and page size must be between 1 and 100");
    }

    let api = ctx.teams_api();
    let result = api.list_teams(Some(page_number), Some(page_size)).await;

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
                "page_number": page_number,
                "page_size": page_size,
            })
        }
    )
}
