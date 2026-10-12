//! Datadog Kubernetes workload-scaling recommendations.

use crate::browser_session::{BrowserSource, CookieSelection, WorkloadRecommendationsQuery};
use crate::response::{CachePolicy, ToolOutput};
use crate::state::ToolContext;
use crate::tool_inputs::{
    BrowserCookieSourceInput, GetWorkloadScalingRecommendationsInput, SortOrderInput,
    WorkloadScalingSortBy,
};
use serde_json::json;
use std::time::Duration;
use tracing::info;

const DEFAULT_PAGE_SIZE: u32 = 25;
const MAX_PAGE_SIZE: u32 = 100;
const MAX_FILTER_LENGTH: usize = 2_048;

impl From<BrowserCookieSourceInput> for BrowserSource {
    fn from(value: BrowserCookieSourceInput) -> Self {
        match value {
            BrowserCookieSourceInput::Chrome => Self::Chrome,
            BrowserCookieSourceInput::Edge => Self::Edge,
            BrowserCookieSourceInput::Firefox => Self::Firefox,
            BrowserCookieSourceInput::Safari => Self::Safari,
        }
    }
}

pub async fn get_workload_scaling_recommendations(
    ctx: ToolContext,
    input: GetWorkloadScalingRecommendationsInput,
) -> anyhow::Result<ToolOutput> {
    let page = input.page.unwrap_or(0);
    let page_size = input.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
        anyhow::bail!("page_size must be between 1 and {MAX_PAGE_SIZE}");
    }

    let query_filter = validate_query_filter(input.query_filter)?;
    let sort_by = input
        .sort_by
        .unwrap_or(WorkloadScalingSortBy::EstimatedSavings)
        .as_query_value()
        .to_string();
    let sort_order = input
        .sort_order
        .unwrap_or(SortOrderInput::Desc)
        .as_query_value()
        .to_string();
    let selection = CookieSelection {
        browser: input
            .browser
            .unwrap_or(BrowserCookieSourceInput::Chrome)
            .into(),
        profile: input.browser_profile,
    };
    let query = WorkloadRecommendationsQuery {
        query_filter,
        page,
        page_size,
        sort_by,
        sort_order,
    };
    let config = ctx.client.config();

    info!(
        site = %config.site,
        browser = %selection.browser,
        page,
        page_size,
        "Getting Datadog workload-scaling recommendations through a local browser session"
    );
    let response = ctx
        .browser_sessions
        .get_workload_recommendations(
            &config.site,
            Duration::from_secs(config.http_config.timeout_secs),
            config.http_config.max_response_bytes,
            selection.clone(),
            &query,
            input.refresh_browser_session.unwrap_or(false),
        )
        .await?;

    let returned_count = response
        .data
        .get("workloads")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let total_count = response
        .data
        .get("total_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(returned_count as u64);

    ToolOutput::from_data(
        &response.data,
        format!("Retrieved {returned_count} of {total_count} workload-scaling recommendations"),
        json!({
            "authentication": "local_browser_session",
            "endpoint_stability": "undocumented_ui",
            "browser": selection.browser.to_string(),
            "browser_profile": selection.profile,
            "cookie_cache_hit": response.cookie_cache_hit,
            "cookie_warning_count": response.cookie_warning_count,
            "returned_count": returned_count,
            "total_count": total_count,
            "page": page,
            "page_size": page_size,
        }),
        CachePolicy::Never,
    )
}

fn validate_query_filter(query_filter: Option<String>) -> anyhow::Result<String> {
    let query_filter = query_filter.unwrap_or_default().trim().to_string();
    if query_filter.len() > MAX_FILTER_LENGTH {
        anyhow::bail!("query_filter cannot exceed {MAX_FILTER_LENGTH} bytes");
    }
    if query_filter.chars().any(char::is_control) {
        anyhow::bail!("query_filter cannot contain control characters");
    }
    Ok(query_filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_filter_is_trimmed_and_bounded() {
        assert_eq!(
            validate_query_filter(Some("  kube_cluster_name:stage  ".to_string())).unwrap(),
            "kube_cluster_name:stage"
        );
        assert!(validate_query_filter(Some("x".repeat(MAX_FILTER_LENGTH + 1))).is_err());
        assert!(validate_query_filter(Some("env:test\nservice:api".to_string())).is_err());
    }

    #[test]
    fn browser_input_maps_to_cookie_source() {
        assert_eq!(
            BrowserSource::from(BrowserCookieSourceInput::Firefox),
            BrowserSource::Firefox
        );
    }
}
