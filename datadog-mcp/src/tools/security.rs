//! Security monitoring tools

use crate::response::ToolOutput;
use crate::state::ToolContext;
use serde_json::json;
use tracing::info;

pub async fn get_security_rules(ctx: ToolContext) -> anyhow::Result<ToolOutput> {
    info!("Getting security monitoring rules");

    let api = ctx.security_api();
    let result = api.list_security_rules().await;

    tool_response_with_fields!(
        result,
        no_cache,
        data,
        {
            let empty_vec = vec![];
            let rules = data.data.as_ref().unwrap_or(&empty_vec);
            let enabled_rules = rules
                .iter()
                .filter(|r| {
                    r.attributes
                        .as_ref()
                        .and_then(|a| a.is_enabled)
                        .unwrap_or(false)
                })
                .count();
            format!(
                "Found {} security rules ({} enabled)",
                rules.len(),
                enabled_rules
            )
        },
        {
            let empty_vec = vec![];
            let rules = data.data.as_ref().unwrap_or(&empty_vec);
            let enabled_rules = rules
                .iter()
                .filter(|r| {
                    r.attributes
                        .as_ref()
                        .and_then(|a| a.is_enabled)
                        .unwrap_or(false)
                })
                .count();
            json!({
                "total_rules": rules.len(),
                "enabled_rules": enabled_rules,
                "disabled_rules": rules.len().saturating_sub(enabled_rules),
            })
        }
    )
}
