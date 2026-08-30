//! Log search tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use datadog_api::models::*;
use serde_json::json;
use tracing::info;

pub async fn search_logs(
    ctx: ToolContext,
    query: String,
    from_time: String,
    to_time: String,
    limit: Option<i32>,
    cursor: Option<String>,
) -> anyhow::Result<ToolOutput> {
    info!(query_length = query.len(), "Searching logs");

    let limit = limit.unwrap_or(100);
    if !(1..=1000).contains(&limit) {
        anyhow::bail!("Log search limit must be between 1 and 1000");
    }

    let request = LogsSearchRequest {
        filter: LogsFilter {
            query,
            from: from_time.clone(),
            to: to_time.clone(),
        },
        page: Some(LogsPage {
            limit: Some(limit),
            cursor,
        }),
        sort: Some("timestamp".to_string()),
    };

    let api = ctx.logs_api();
    let result = api.search_logs(&request).await;

    tool_response_with_fields!(
        result,
        no_cache,
        data,
        {
            let log_count = data.data.as_ref().map(|l| l.len()).unwrap_or(0);
            format!("Retrieved {} log entries", log_count)
        },
        {
            let logs = data.data.as_ref().map(|l| l.len()).unwrap_or(0);
            json!({
                "log_count": logs,
                "next_cursor": data.meta.as_ref()
                    .and_then(|meta| meta.page.as_ref())
                    .and_then(|page| page.after.clone()),
                "time_range": format!("{} to {}", from_time, to_time),
            })
        }
    )
}
