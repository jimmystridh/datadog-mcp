//! API validation tools

use crate::state::ToolContext;
use serde_json::{json, Value};
use tracing::{error, info};

pub async fn validate_api_key(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Validating API credentials");

    let api = ctx.monitors_api();
    let result = api.list_monitors_with_page_size(1).await;

    match result {
        Ok(_) => {
            info!("API credentials validated successfully");
            Ok(json!({
                "valid": true,
                "summary": "API credentials are valid and working",
                "site": ctx.client.config().site,
                "test_successful": true,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("API validation failed: {}", e);
            Ok(json!({
                "valid": false,
                "error": format!("API validation failed: {}", e),
                "site": ctx.client.config().site,
                "status": "error",
            }))
        }
    }
}
