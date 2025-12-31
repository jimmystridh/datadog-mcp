//! Notebook tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::info;

pub async fn get_notebooks(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting Datadog notebooks");

    let api = ctx.notebooks_api();
    let result = api.list_notebooks().await;

    tool_response_with_fields!(
        result,
        "notebooks",
        ctx,
        data,
        {
            let notebooks = data.data.as_ref().map(|n| n.len()).unwrap_or(0);
            format!("Retrieved {} notebooks", notebooks)
        },
        {
            let notebooks = data.data.as_ref().map(|n| n.len()).unwrap_or(0);
            json!({
                "total_notebooks": notebooks,
            })
        }
    )
}
