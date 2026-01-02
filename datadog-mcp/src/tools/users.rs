//! User tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_users(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting users");

    let api = ctx.users_api();
    let result = api.list_users().await;

    tool_response_with_fields!(
        result,
        "users",
        ctx,
        data,
        {
            let users = data.users.as_ref().map(|u| u.len()).unwrap_or(0);
            format!("Retrieved {} users", users)
        },
        {
            let users = data.users.as_ref().map(|u| u.len()).unwrap_or(0);
            json!({
                "total_users": users,
            })
        }
    )
}
