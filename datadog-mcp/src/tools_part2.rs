use crate::cache::{cleanup_cache, load_data, store_data};
use datadog_api::{apis::*, models::*, DatadogClient};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

// ============================================================================
// LOGS & EVENTS TOOLS
// ============================================================================

pub async fn search_logs(
    ctx: Arc<DatadogClient>,
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

    let api = LogsApi::new((*ctx).clone());
    let result = api.search_logs(&request).await;

    match result {
        Ok(data) => {
            let logs = data.data.as_ref().map(|l| l.len()).unwrap_or(0);
            let filepath = store_data(&data, "logs").await?;
            info!("Retrieved {} log entries", logs);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} log entries", logs),
                "log_count": logs,
                "time_range": format!("{} to {}", from_time, to_time),
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to search logs: {}", e);
            Ok(json!({
                "error": format!("Failed to search logs: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_events(
    ctx: Arc<DatadogClient>,
    start: i64,
    end: i64,
    priority: Option<String>,
    sources: Option<String>,
) -> anyhow::Result<Value> {
    info!("Getting events from {} to {}", start, end);

    let api = EventsApi::new((*ctx).clone());
    let result = api
        .list_events(start, end, priority.as_deref(), sources.as_deref())
        .await;

    match result {
        Ok(data) => {
            let events = data.events.as_ref().map(|e| e.len()).unwrap_or(0);
            let filepath = store_data(&data, "events").await?;
            info!("Retrieved {} events", events);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} events", events),
                "event_count": events,
                "time_range": format!("{} to {}", start, end),
                "priority_filter": priority,
                "sources_filter": sources,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get events: {}", e);
            Ok(json!({
                "error": format!("Failed to get events: {}", e),
                "status": "error",
            }))
        }
    }
}

// ============================================================================
// INFRASTRUCTURE & TAGS TOOLS
// ============================================================================

pub async fn get_infrastructure(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting infrastructure information");

    let api = InfrastructureApi::new((*ctx).clone());
    let result = api.list_hosts().await;

    match result {
        Ok(data) => {
            let hosts = data.host_list.clone().unwrap_or_default();
            let active_hosts = hosts.iter().filter(|h| h.up.unwrap_or(false)).count();
            let total_hosts = hosts.len();

            let filepath = store_data(&data, "infrastructure").await?;
            info!("Found {} hosts ({} active)", total_hosts, active_hosts);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Found {} hosts ({} active)", total_hosts, active_hosts),
                "total_hosts": total_hosts,
                "active_hosts": active_hosts,
                "inactive_hosts": total_hosts - active_hosts,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get infrastructure: {}", e);
            Ok(json!({
                "error": format!("Failed to get infrastructure: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_tags(ctx: Arc<DatadogClient>, source: Option<String>) -> anyhow::Result<Value> {
    info!("Getting host tags");

    let api = InfrastructureApi::new((*ctx).clone());
    let result = api.get_tags(source.as_deref()).await;

    match result {
        Ok(data) => {
            let tags = data.tags.as_ref().map(|t| t.len()).unwrap_or(0);
            let filepath = store_data(&data, "tags").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved tags for {} hosts", tags),
                "host_count": tags,
                "source": source.unwrap_or_else(|| "all".to_string()),
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get tags: {}", e);
            Ok(json!({
                "error": format!("Failed to get tags: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_downtimes(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting scheduled downtimes");

    let api = DowntimesApi::new((*ctx).clone());
    let result = api.list_downtimes().await;

    match result {
        Ok(data) => {
            let active_count = data.iter().filter(|d| d.active.unwrap_or(false)).count();
            let filepath = store_data(&data, "downtimes").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} downtimes ({} active)", data.len(), active_count),
                "total_downtimes": data.len(),
                "active_downtimes": active_count,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get downtimes: {}", e);
            Ok(json!({
                "error": format!("Failed to get downtimes: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn create_downtime(
    ctx: Arc<DatadogClient>,
    scope: Vec<String>,
    start: Option<i64>,
    end: Option<i64>,
    message: Option<String>,
) -> anyhow::Result<Value> {
    info!("Creating downtime for scope: {:?}", scope);

    let request = DowntimeCreateRequest {
        scope: scope.clone(),
        start,
        end,
        message,
    };

    let api = DowntimesApi::new((*ctx).clone());
    let result = api.create_downtime(&request).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "downtime_created").await?;
            info!("Created downtime (ID: {:?})", data.id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Created downtime (ID: {:?})", data.id),
                "downtime_id": data.id,
                "scope": data.scope,
                "status": "created",
            }))
        }
        Err(e) => {
            error!("Failed to create downtime: {}", e);
            Ok(json!({
                "error": format!("Failed to create downtime: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn cancel_downtime(ctx: Arc<DatadogClient>, downtime_id: i64) -> anyhow::Result<Value> {
    info!("Cancelling downtime ID: {}", downtime_id);

    let api = DowntimesApi::new((*ctx).clone());
    let result = api.cancel_downtime(downtime_id).await;

    match result {
        Ok(()) => {
            info!("Cancelled downtime ID: {}", downtime_id);
            Ok(json!({
                "summary": format!("Cancelled downtime ID: {}", downtime_id),
                "downtime_id": downtime_id,
                "status": "cancelled"
            }))
        }
        Err(e) => {
            error!("Failed to cancel downtime: {}", e);
            Ok(json!({
                "error": format!("Failed to cancel downtime: {}", e),
                "status": "error",
            }))
        }
    }
}

// ============================================================================
// TESTING & APPLICATIONS TOOLS
// ============================================================================

pub async fn get_synthetics_tests(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting Synthetics tests");

    let api = SyntheticsApi::new((*ctx).clone());
    let result = api.list_tests().await;

    match result {
        Ok(data) => {
            let empty_vec = vec![];
            let tests = data.tests.as_ref().unwrap_or(&empty_vec);
            let mut test_types: HashMap<String, usize> = HashMap::new();

            for test in tests {
                if let Some(test_type) = &test.test_type {
                    *test_types.entry(test_type.clone()).or_insert(0) += 1;
                }
            }

            let filepath = store_data(&data, "synthetics_tests").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Found {} Synthetics tests", tests.len()),
                "test_count": tests.len(),
                "test_types": test_types,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get Synthetics tests: {}", e);
            Ok(json!({
                "error": format!("Failed to get Synthetics tests: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_synthetics_locations(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting Synthetics locations");

    let api = SyntheticsApi::new((*ctx).clone());
    let result = api.list_locations().await;

    match result {
        Ok(data) => {
            let public_locs: Vec<_> = data
                .locations
                .iter()
                .filter(|loc| !loc.is_private.unwrap_or(false))
                .collect();
            let private_locs: Vec<_> = data
                .locations
                .iter()
                .filter(|loc| loc.is_private.unwrap_or(false))
                .collect();

            let mut regions: HashMap<String, Vec<String>> = HashMap::new();
            for loc in &public_locs {
                let region_name = loc
                    .region
                    .as_ref()
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                regions
                    .entry(region_name)
                    .or_default()
                    .push(loc.id.clone());
            }

            let eu_locations: Vec<String> = public_locs
                .iter()
                .filter(|loc| loc.id.to_lowercase().contains("eu"))
                .map(|loc| loc.id.clone())
                .collect();

            let filepath = store_data(&data, "synthetics_locations").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!(
                    "Retrieved {} Synthetics locations ({} public, {} private)",
                    data.locations.len(),
                    public_locs.len(),
                    private_locs.len()
                ),
                "total_locations": data.locations.len(),
                "public_count": public_locs.len(),
                "private_count": private_locs.len(),
                "regions": regions,
                "eu_locations": eu_locations,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get Synthetics locations: {}", e);
            Ok(json!({
                "error": format!("Failed to get Synthetics locations: {}", e),
                "status": "error",
            }))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_synthetics_test(
    ctx: Arc<DatadogClient>,
    name: String,
    test_type: String,
    url: String,
    locations: Vec<String>,
    message: Option<String>,
    tags: Option<Vec<String>>,
    tick_every: Option<i32>,
) -> anyhow::Result<Value> {
    use datadog_api::models::*;

    info!("Creating Synthetics test: {}", name);

    // Validate test_type
    if test_type != "api" {
        return Ok(json!({
            "error": "Only 'api' test type is currently supported",
            "status": "error",
        }));
    }

    // Build test request
    let request = SyntheticsTestCreateRequest {
        name: name.clone(),
        test_type: SyntheticsTestType::Api,
        subtype: SyntheticsTestSubtype::Http,
        config: SyntheticsTestConfig {
            request: SyntheticsTestRequest {
                method: "GET".to_string(),
                url: url.clone(),
                timeout: Some(30.0),
                headers: None,
                body: None,
            },
            assertions: vec![
                SyntheticsAssertion {
                    assertion_type: SyntheticsAssertionType::StatusCode,
                    operator: SyntheticsAssertionOperator::Is,
                    target: json!(200),
                },
                SyntheticsAssertion {
                    assertion_type: SyntheticsAssertionType::ResponseTime,
                    operator: SyntheticsAssertionOperator::LessThan,
                    target: json!(3000),
                },
            ],
        },
        options: SyntheticsTestOptions {
            tick_every: tick_every.unwrap_or(300),
            min_failure_duration: Some(0),
            min_location_failed: Some(1),
            retry: Some(SyntheticsRetry {
                count: 2,
                interval: 300,
            }),
        },
        locations,
        message,
        tags,
        status: Some("live".to_string()),
    };

    let api = SyntheticsApi::new((*ctx).clone());
    let result = api.create_test(&request).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "synthetics_test_created").await?;
            info!(
                "Created Synthetics test: {} (ID: {})",
                data.name, data.public_id
            );

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Created Synthetics test: {}", data.name),
                "public_id": data.public_id,
                "name": data.name,
                "type": data.test_type,
                "status": data.status,
                "url": url,
            }))
        }
        Err(e) => {
            error!("Failed to create Synthetics test: {}", e);
            Ok(json!({
                "error": format!("Failed to create Synthetics test: {}", e),
                "status": "error",
            }))
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn update_synthetics_test(
    ctx: Arc<DatadogClient>,
    public_id: String,
    name: Option<String>,
    url: Option<String>,
    locations: Option<Vec<String>>,
    message: Option<String>,
    tags: Option<Vec<String>>,
    tick_every: Option<i32>,
) -> anyhow::Result<Value> {
    use datadog_api::models::*;

    info!("Updating Synthetics test: {}", public_id);

    let api = SyntheticsApi::new((*ctx).clone());

    // Get the existing test
    let existing = api.get_test(&public_id).await.map_err(|e| {
        error!("Failed to get existing Synthetics test: {}", e);
        anyhow::anyhow!("Failed to get existing test: {}", e)
    })?;

    // Merge updates with existing configuration
    let updated_config = if let Some(new_url) = url {
        SyntheticsTestConfig {
            request: SyntheticsTestRequest {
                url: new_url,
                method: existing.config.request.method.clone(),
                timeout: existing.config.request.timeout,
                headers: existing.config.request.headers.clone(),
                body: existing.config.request.body.clone(),
            },
            assertions: existing.config.assertions.clone(),
        }
    } else {
        existing.config.clone()
    };

    let updated_request = SyntheticsTestCreateRequest {
        name: name.unwrap_or(existing.name),
        test_type: existing.test_type,
        subtype: existing.subtype,
        config: updated_config,
        options: SyntheticsTestOptions {
            tick_every: tick_every.unwrap_or(existing.options.tick_every),
            retry: existing.options.retry,
            min_failure_duration: existing.options.min_failure_duration,
            min_location_failed: existing.options.min_location_failed,
        },
        locations: locations.unwrap_or(existing.locations),
        message: message.or(existing.message),
        tags: tags.or(existing.tags),
        status: Some(existing.status),
    };

    // Send the update
    let result = api.update_test(&public_id, &updated_request).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "synthetics_test_updated").await?;
            info!("Updated Synthetics test: {} (ID: {})", data.name, data.public_id);

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Updated Synthetics test: {}", data.name),
                "public_id": data.public_id,
                "name": data.name,
                "status": data.status,
            }))
        }
        Err(e) => {
            error!("Failed to update Synthetics test: {}", e);
            Ok(json!({
                "error": format!("Failed to update Synthetics test: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn trigger_synthetics_tests(
    ctx: Arc<DatadogClient>,
    test_ids: Vec<String>,
) -> anyhow::Result<Value> {
    info!("Triggering {} Synthetics test(s)", test_ids.len());

    if test_ids.is_empty() {
        return Ok(json!({
            "error": "At least one test ID must be provided",
            "status": "error",
        }));
    }

    let api = SyntheticsApi::new((*ctx).clone());
    let result = api.trigger_tests(test_ids.clone()).await;

    match result {
        Ok(data) => {
            let filepath = store_data(&data, "synthetics_tests_triggered").await?;
            info!(
                "Triggered {} test(s), {} check(s) started",
                test_ids.len(),
                data.triggered_check_ids.len()
            );

            Ok(json!({
                "filepath": filepath,
                "summary": format!(
                    "Triggered {} test(s), {} check(s) started",
                    test_ids.len(),
                    data.triggered_check_ids.len()
                ),
                "test_ids": test_ids,
                "triggered_check_ids": data.triggered_check_ids,
                "results": data.results,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to trigger Synthetics tests: {}", e);
            Ok(json!({
                "error": format!("Failed to trigger Synthetics tests: {}", e),
                "status": "error",
            }))
        }
    }
}

// ============================================================================
// SECURITY & INCIDENTS TOOLS
// ============================================================================

pub async fn get_security_rules(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting security monitoring rules");

    let api = SecurityApi::new((*ctx).clone());
    let result = api.list_security_rules().await;

    match result {
        Ok(data) => {
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

            let filepath = store_data(&data, "security_rules").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Found {} security rules ({} enabled)", rules.len(), enabled_rules),
                "total_rules": rules.len(),
                "enabled_rules": enabled_rules,
                "disabled_rules": rules.len() - enabled_rules,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get security rules: {}", e);
            Ok(json!({
                "error": format!("Failed to get security rules: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_incidents(
    ctx: Arc<DatadogClient>,
    page_size: Option<i32>,
) -> anyhow::Result<Value> {
    info!("Getting incidents");

    let api = IncidentsApi::new((*ctx).clone());
    let result = api.list_all_incidents(page_size.unwrap_or(25)).await;

    match result {
        Ok(incidents) => {
            let mut states: HashMap<String, usize> = HashMap::new();

            for incident in &incidents {
                if let Some(attrs) = &incident.attributes {
                    if let Some(state) = &attrs.state {
                        *states.entry(state.clone()).or_insert(0) += 1;
                    }
                }
            }

            let filepath = store_data(&incidents, "incidents").await?;
            info!("Retrieved {} incidents", incidents.len());

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} incidents", incidents.len()),
                "total_incidents": incidents.len(),
                "incident_states": states,
                "active_incidents": states.get("active").unwrap_or(&0),
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get incidents: {}", e);
            Ok(json!({
                "error": format!("Failed to get incidents: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_slos(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting Service Level Objectives");

    let api = SLOsApi::new((*ctx).clone());
    let result = api.list_slos().await;

    match result {
        Ok(data) => {
            let slos = data.data.as_ref().map(|s| s.len()).unwrap_or(0);
            let filepath = store_data(&data, "slos").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} SLOs", slos),
                "total_slos": slos,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get SLOs: {}", e);
            Ok(json!({
                "error": format!("Failed to get SLOs: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_notebooks(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting Datadog notebooks");

    let api = NotebooksApi::new((*ctx).clone());
    let result = api.list_notebooks().await;

    match result {
        Ok(data) => {
            let notebooks = data.data.as_ref().map(|n| n.len()).unwrap_or(0);
            let filepath = store_data(&data, "notebooks").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} notebooks", notebooks),
                "total_notebooks": notebooks,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get notebooks: {}", e);
            Ok(json!({
                "error": format!("Failed to get notebooks: {}", e),
                "status": "error",
            }))
        }
    }
}

// ============================================================================
// TEAMS & USERS TOOLS
// ============================================================================

pub async fn get_teams(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting teams");

    let api = TeamsApi::new((*ctx).clone());
    let result = api.list_teams().await;

    match result {
        Ok(data) => {
            let teams = data.data.as_ref().map(|t| t.len()).unwrap_or(0);
            let filepath = store_data(&data, "teams").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} teams", teams),
                "total_teams": teams,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get teams: {}", e);
            Ok(json!({
                "error": format!("Failed to get teams: {}", e),
                "status": "error",
            }))
        }
    }
}

pub async fn get_users(ctx: Arc<DatadogClient>) -> anyhow::Result<Value> {
    info!("Getting users");

    let api = UsersApi::new((*ctx).clone());
    let result = api.list_users().await;

    match result {
        Ok(data) => {
            let users = data.users.as_ref().map(|u| u.len()).unwrap_or(0);
            let filepath = store_data(&data, "users").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!("Retrieved {} users", users),
                "total_users": users,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get users: {}", e);
            Ok(json!({
                "error": format!("Failed to get users: {}", e),
                "status": "error",
            }))
        }
    }
}

// ============================================================================
// UTILITY TOOLS
// ============================================================================

pub async fn analyze_data(
    filepath: String,
    analysis_type: Option<String>,
) -> anyhow::Result<Value> {
    info!("Analyzing data from: {}", filepath);

    let data = load_data(&filepath).await?;
    let analysis = analysis_type.unwrap_or_else(|| "summary".to_string());

    let result = match analysis.as_str() {
        "summary" => generate_summary(&data),
        "stats" => calculate_stats(&data),
        "trends" => analyze_trends(&data),
        _ => {
            return Ok(json!({
                "error": format!("Unknown analysis type: {}", analysis),
                "status": "error",
            }));
        }
    };

    info!("Completed {} analysis of {}", analysis, filepath);

    Ok(json!({
        "analysis_type": analysis,
        "filepath": filepath,
        "result": result,
        "status": "success",
    }))
}

pub async fn cleanup_cache_tool(older_than_hours: Option<u64>) -> anyhow::Result<Value> {
    let hours = older_than_hours.unwrap_or(24);
    info!("Cleaning up cache files older than {} hours", hours);

    match cleanup_cache(hours).await {
        Ok(deleted_count) => {
            info!(
                "Cleaned up {} files older than {} hours",
                deleted_count, hours
            );
            Ok(json!({
                "summary": format!("Cleaned up {} files older than {} hours", deleted_count, hours),
                "deleted_count": deleted_count,
                "cache_directory": "datadog_cache",
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to cleanup cache: {}", e);
            Ok(json!({
                "error": format!("Failed to cleanup cache: {}", e),
                "status": "error",
            }))
        }
    }
}

// Analysis helper functions

fn generate_summary(data: &Value) -> Value {
    let mut summary = json!({
        "data_type": "unknown",
        "record_count": 0,
        "key_insights": []
    });

    if let Some(obj) = data.as_object() {
        if obj.contains_key("series") {
            if let Some(series) = obj["series"].as_array() {
                summary["data_type"] = json!("metrics");
                summary["record_count"] = json!(series.len());

                let total_points: usize = series
                    .iter()
                    .filter_map(|s| s["pointlist"].as_array())
                    .map(|p| p.len())
                    .sum();

                summary["total_data_points"] = json!(total_points);

                if total_points > 1000 {
                    if let Some(insights) = summary["key_insights"].as_array_mut() {
                        insights.push(json!("Large dataset - consider aggregation"));
                    }
                }
            }
        } else if let Some(arr) = data.as_array() {
            if !arr.is_empty() {
                if let Some(first) = arr[0].as_object() {
                    if first.contains_key("overall_state") {
                        summary["data_type"] = json!("monitors");
                        summary["record_count"] = json!(arr.len());

                        let alerting = arr
                            .iter()
                            .filter(|m| m["overall_state"].as_str() == Some("Alert"))
                            .count();

                        summary["alerting_monitors"] = json!(alerting);

                        if alerting > 0 {
                            if let Some(insights) = summary["key_insights"].as_array_mut() {
                                insights.push(json!(format!(
                                    "{} monitors currently alerting",
                                    alerting
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    summary
}

fn calculate_stats(data: &Value) -> Value {
    let mut stats = json!({
        "calculated_at": chrono::Utc::now().to_rfc3339()
    });

    if let Some(obj) = data.as_object() {
        if let Some(series) = obj.get("series").and_then(|s| s.as_array()) {
            let mut all_values = Vec::new();

            for s in series {
                if let Some(pointlist) = s["pointlist"].as_array() {
                    for point in pointlist {
                        if let Some(arr) = point.as_array() {
                            if arr.len() >= 2 {
                                if let Some(val) = arr[1].as_f64() {
                                    all_values.push(val);
                                }
                            }
                        }
                    }
                }
            }

            if !all_values.is_empty() {
                let min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = all_values.iter().sum();
                let avg = sum / all_values.len() as f64;

                stats["min_value"] = json!(min);
                stats["max_value"] = json!(max);
                stats["avg_value"] = json!(avg);
                stats["total_points"] = json!(all_values.len());
            }
        }
    }

    stats
}

fn analyze_trends(data: &Value) -> Value {
    let mut trends = json!({
        "trend_direction": "stable",
        "analyzed_at": chrono::Utc::now().to_rfc3339()
    });

    if let Some(obj) = data.as_object() {
        if let Some(series) = obj.get("series").and_then(|s| s.as_array()) {
            if let Some(first_series) = series.first() {
                if let Some(pointlist) = first_series["pointlist"].as_array() {
                    let values: Vec<f64> = pointlist
                        .iter()
                        .filter_map(|p| p.as_array())
                        .filter_map(|arr| {
                            if arr.len() >= 2 {
                                arr[1].as_f64()
                            } else {
                                None
                            }
                        })
                        .collect();

                    if values.len() >= 2 {
                        let first_val = values[0];
                        let last_val = values[values.len() - 1];

                        if first_val != 0.0 {
                            let change_pct = ((last_val - first_val) / first_val) * 100.0;
                            trends["change_percentage"] = json!(format!("{:.2}", change_pct));

                            if change_pct > 10.0 {
                                trends["trend_direction"] = json!("increasing");
                            } else if change_pct < -10.0 {
                                trends["trend_direction"] = json!("decreasing");
                            }
                        }
                    }
                }
            }
        }
    }

    trends
}

// ============================================================================
// KUBERNETES HELPER TOOLS
// ============================================================================

pub async fn get_kubernetes_deployments(
    ctx: Arc<DatadogClient>,
    namespace: Option<String>,
) -> anyhow::Result<Value> {
    use std::time::{SystemTime, UNIX_EPOCH};

    info!(
        "Getting Kubernetes deployments{}",
        namespace
            .as_ref()
            .map(|ns| format!(" in namespace: {}", ns))
            .unwrap_or_default()
    );

    // Query for deployment replicas in the last 5 minutes
    let to_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let from_ts = to_ts - 300; // 5 minutes ago

    // Build query with optional namespace filter
    let namespace_filter = namespace
        .as_ref()
        .map(|ns| format!(",kube_namespace:{}", ns))
        .unwrap_or_default();

    let query = format!(
        "avg:kubernetes_state.deployment.replicas_desired{{*{}}} by {{kube_deployment,kube_namespace,kube_cluster_name}}",
        namespace_filter
    );

    // Use existing metrics API
    let api = MetricsApi::new((*ctx).clone());
    let result = api.query_metrics(from_ts, to_ts, &query).await;

    match result {
        Ok(data) => {
            // Extract deployment information from series
            let mut deployments = Vec::new();
            let mut unique_deployment_names = std::collections::HashSet::new();
            let mut unique_namespaces = std::collections::HashSet::new();

            if let Some(series) = &data.series {
                for s in series {
                    // Parse tags from scope
                    let mut tags = std::collections::HashMap::new();
                    if let Some(scope) = &s.scope {
                        for tag in scope.split(',') {
                            if let Some((key, value)) = tag.split_once(':') {
                                tags.insert(key.to_string(), value.to_string());
                            }
                        }
                    }

                    let deployment = tags
                        .get("kube_deployment")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let ns = tags
                        .get("kube_namespace")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());
                    let cluster = tags
                        .get("kube_cluster_name")
                        .cloned()
                        .unwrap_or_else(|| "unknown".to_string());

                    unique_deployment_names.insert(deployment.clone());
                    unique_namespaces.insert(ns.clone());

                    deployments.push(json!({
                        "deployment": deployment,
                        "namespace": ns,
                        "cluster": cluster,
                    }));
                }
            }

            let mut deployment_names: Vec<String> = unique_deployment_names.into_iter().collect();
            deployment_names.sort();

            let mut namespace_list: Vec<String> = unique_namespaces.into_iter().collect();
            namespace_list.sort();

            let filepath = store_data(&data, "kubernetes_deployments").await?;

            Ok(json!({
                "filepath": filepath,
                "summary": format!(
                    "Found {} deployments across {} namespaces",
                    deployment_names.len(),
                    namespace_list.len()
                ),
                "deployments": deployments,
                "unique_deployment_names": deployment_names,
                "unique_namespaces": namespace_list,
                "status": "success",
            }))
        }
        Err(e) => {
            error!("Failed to get Kubernetes deployments: {}", e);
            Ok(json!({
                "error": format!("Failed to get Kubernetes deployments: {}", e),
                "status": "error",
            }))
        }
    }
}
