//! User tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_users(
    ctx: ToolContext,
    page_number: Option<i64>,
    page_size: Option<i64>,
) -> anyhow::Result<Value> {
    info!("Getting users");

    let page_number = page_number.unwrap_or(1);
    let page_size = page_size.unwrap_or(100);
    if page_number < 1 || !(1..=100).contains(&page_size) {
        anyhow::bail!("User page number must be positive and page size must be between 1 and 100");
    }

    let api = ctx.users_api();
    let result = api.list_users(Some(page_number), Some(page_size)).await;

    tool_response_with_fields!(
        result,
        "users",
        ctx,
        data,
        {
            let users = data.data.as_ref().map(|u| u.len()).unwrap_or(0);
            format!("Retrieved {} users", users)
        },
        {
            let users = data.data.as_ref().map(|u| u.len()).unwrap_or(0);
            json!({
                "total_users": users,
                "page_number": page_number,
                "page_size": page_size,
            })
        }
    )
}
