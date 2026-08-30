//! API validation tools

use crate::response::{CachePolicy, ToolOutput};
use crate::state::ToolContext;
use serde_json::json;
use tracing::info;

pub async fn validate_api_key(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Validating API credentials");

    ctx.client.validate_keys().await?;
    info!("API credentials validated successfully");
    ToolOutput::from_data(
        &json!({ "valid": true }),
        "API credentials are valid and working",
        json!({
            "site": ctx.client.config().site,
            "test_successful": true,
        }),
        CachePolicy::Never,
    )
}
