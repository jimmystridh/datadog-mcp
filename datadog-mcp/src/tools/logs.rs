//! Log search tools

use crate::state::ToolContext;
use datadog_api::models::*;
use serde_json::{json, Value};
use tracing::info;

pub async fn search_logs(
    ctx: ToolContext,
    query: String,
    from_time: String,
    to_time: String,
    limit: Option<i32>,
) -> anyhow::Result<Value> {
    info!("Searching logs with query: {}", query);

    let request = LogsSearchRequest {
        filter: LogsFilter {
            query,
            from: from_time.clone(),
            to: to_time.clone(),
        },
        page: Some(LogsPage {
            limit,
            cursor: None,
        }),
        sort: Some("timestamp".to_string()),
    };

    let api = ctx.logs_api();
    let result = api.search_logs(&request).await;

    tool_response_with_fields!(
        result,
        "logs",
        ctx,
        data,
        {
            let log_count = data.data.as_ref().map(|l| l.len()).unwrap_or(0);
            format!("Retrieved {} log entries", log_count)
        },
        {
            let logs = data.data.as_ref().map(|l| l.len()).unwrap_or(0);
            json!({
                "log_count": logs,
                "time_range": format!("{} to {}", from_time, to_time),
            })
        }
    )
}
