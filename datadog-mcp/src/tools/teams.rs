//! Team tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use datadog_api::NumberedPage;
use serde_json::json;
use tracing::info;

pub async fn get_teams(
    ctx: ToolContext,
    page_number: Option<i64>,
    page_size: Option<i64>,
) -> anyhow::Result<ToolOutput> {
    info!("Getting teams");

    let page = NumberedPage::new(page_number.unwrap_or(1), page_size.unwrap_or(100))?;

    let api = ctx.teams_api();
    let result = api.list_teams(page).await;

    tool_response_with_fields!(
        result,
        cache("teams"),
        data,
        {
            let teams = data.data.as_ref().map(|t| t.len()).unwrap_or(0);
            format!("Retrieved {} teams", teams)
        },
        {
            let teams = data.data.as_ref().map(|t| t.len()).unwrap_or(0);
            json!({
                "total_teams": teams,
                "page_number": page.number(),
                "page_size": page.size(),
            })
        }
    )
}
