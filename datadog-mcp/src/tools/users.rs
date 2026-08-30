//! User tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use datadog_api::NumberedPage;
use serde_json::json;
use tracing::info;

pub async fn get_users(
    ctx: ToolContext,
    page_number: Option<i64>,
    page_size: Option<i64>,
) -> anyhow::Result<ToolOutput> {
    info!("Getting users");

    let page = NumberedPage::new(page_number.unwrap_or(1), page_size.unwrap_or(100))?;

    let api = ctx.users_api();
    let result = api.list_users(page).await;

    tool_response_with_fields!(
        result,
        no_cache,
        data,
        {
            let users = data.data.as_ref().map(|u| u.len()).unwrap_or(0);
            format!("Retrieved {} users", users)
        },
        {
            let users = data.data.as_ref().map(|u| u.len()).unwrap_or(0);
            json!({
                "total_users": users,
                "page_number": page.number(),
                "page_size": page.size(),
            })
        }
    )
}
