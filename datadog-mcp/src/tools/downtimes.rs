//! Downtime tools

use crate::ids::DowntimeId;
use crate::response::{simple_success_with_fields, ToolOutput};
use crate::sanitize::{sanitize_optional, sanitize_tags, MAX_MESSAGE_LENGTH};
use crate::state::ToolContext;
use datadog_api::models::*;
use serde_json::json;
use tracing::info;

pub async fn get_downtimes(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Getting scheduled downtimes");

    let api = ctx.downtimes_api();
    let result = api.list_downtimes().await;

    tool_response_with_fields!(
        result,
        cache("downtimes"),
        data,
        {
            let active_count = data.iter().filter(|d| d.active.unwrap_or(false)).count();
            format!(
                "Retrieved {} downtimes ({} active)",
                data.len(),
                active_count
            )
        },
        {
            let active_count = data.iter().filter(|d| d.active.unwrap_or(false)).count();
            json!({
                "total_downtimes": data.len(),
                "active_downtimes": active_count,
            })
        }
    )
}

pub async fn create_downtime(
    ctx: ToolContext,
    scope: Vec<String>,
    start: Option<i64>,
    end: Option<i64>,
    message: Option<String>,
) -> anyhow::Result<ToolOutput> {
    let scope = sanitize_tags(scope);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);

    info!("Creating downtime for scope: {:?}", scope);

    let request = DowntimeCreateRequest {
        scope: scope.clone(),
        start,
        end,
        message,
    };

    let api = ctx.downtimes_api();
    let result = api.create_downtime(&request).await;

    tool_response_with_fields!(
        result,
        cache("downtime_created"),
        data,
        format!("Created downtime (ID: {:?})", data.id),
        {
            json!({
                "downtime_id": data.id,
                "scope": data.scope,
                "operation_status": "created",
            })
        }
    )
}

pub async fn cancel_downtime(
    ctx: ToolContext,
    downtime_id: DowntimeId,
) -> anyhow::Result<ToolOutput> {
    info!("Cancelling downtime ID: {}", downtime_id);

    let api = ctx.downtimes_api();
    api.cancel_downtime(downtime_id.0).await?;
    info!("Cancelled downtime ID: {}", downtime_id);
    simple_success_with_fields(
        format!("Cancelled downtime ID: {}", downtime_id),
        json!({
            "downtime_id": downtime_id,
            "operation_status": "cancelled"
        }),
    )
}
