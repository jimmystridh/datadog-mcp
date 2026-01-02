//! Synthetics testing tools

use crate::ids::SyntheticsTestId;
use crate::sanitize::{sanitize_name, sanitize_optional, sanitize_tags, MAX_MESSAGE_LENGTH, MAX_NAME_LENGTH};
use crate::state::ToolContext;
use datadog_api::models::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{error, info};

pub async fn get_synthetics_tests(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting Synthetics tests");

    let api = ctx.synthetics_api();
    let result = api.list_tests().await;

    tool_response_with_fields!(
        result,
        "synthetics_tests",
        ctx,
        data,
        {
            let empty_vec = vec![];
            let tests = data.tests.as_ref().unwrap_or(&empty_vec);
            format!("Found {} Synthetics tests", tests.len())
        },
        {
            let empty_vec = vec![];
            let tests = data.tests.as_ref().unwrap_or(&empty_vec);
            let mut test_types: HashMap<String, usize> = HashMap::new();
            for test in tests {
                if let Some(test_type) = &test.test_type {
                    *test_types.entry(test_type.clone()).or_insert(0) += 1;
                }
            }
            json!({
                "test_count": tests.len(),
                "test_types": test_types,
            })
        }
    )
}

pub async fn delete_synthetics_tests(
    ctx: ToolContext,
    test_ids: Vec<SyntheticsTestId>,
    force_delete_dependencies: Option<bool>,
) -> anyhow::Result<Value> {
    info!("Deleting {} Synthetics test(s)", test_ids.len());

    if test_ids.is_empty() {
        return Ok(json!({
            "error": "At least one test ID must be provided",
            "status": "error",
        }));
    }

    let api = ctx.synthetics_api();
    let ids: Vec<String> = test_ids.iter().map(|id| id.0.clone()).collect();
    let request = SyntheticsDeleteTestsRequest {
        force_delete_dependencies,
        public_ids: ids,
    };

    let result = api.delete_tests(&request).await;

    tool_response_with_fields!(
        result,
        "synthetics_tests_deleted",
        ctx,
        data,
        {
            let deleted = data.deleted_tests.as_ref().map(|d| d.len()).unwrap_or(0);
            format!("Deleted {} Synthetics test(s)", deleted)
        },
        {
            let deleted = data.deleted_tests.as_ref().map(|d| d.len()).unwrap_or(0);
            json!({
                "requested": test_ids.len(),
                "deleted": deleted,
                "force_delete_dependencies": force_delete_dependencies,
            })
        }
    )
}

pub async fn get_synthetics_locations(ctx: ToolContext) -> anyhow::Result<Value> {
    info!("Getting Synthetics locations");

    let api = ctx.synthetics_api();
    let result = api.list_locations().await;

    tool_response_with_fields!(
        result,
        "synthetics_locations",
        ctx,
        data,
        {
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
            format!(
                "Retrieved {} Synthetics locations ({} public, {} private)",
                data.locations.len(),
                public_locs.len(),
                private_locs.len()
            )
        },
        {
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
                regions.entry(region_name).or_default().push(loc.id.clone());
            }

            let eu_locations: Vec<String> = public_locs
                .iter()
                .filter(|loc| loc.id.to_lowercase().contains("eu"))
                .map(|loc| loc.id.clone())
                .collect();

            json!({
                "total_locations": data.locations.len(),
                "public_count": public_locs.len(),
                "private_count": private_locs.len(),
                "regions": regions,
                "eu_locations": eu_locations,
            })
        }
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn create_synthetics_test(
    ctx: ToolContext,
    name: String,
    test_type: String,
    url: String,
    locations: Vec<String>,
    message: Option<String>,
    tags: Option<Vec<String>>,
    tick_every: Option<i32>,
) -> anyhow::Result<Value> {
    let name = sanitize_name(&name);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);
    let tags = tags.map(sanitize_tags);

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

    let api = ctx.synthetics_api();
    let result = api.create_test(&request).await;

    tool_response_with_fields!(
        result,
        "synthetics_test_created",
        ctx,
        data,
        format!("Created Synthetics test: {}", data.name),
        {
            json!({
                "public_id": data.public_id,
                "name": data.name,
                "type": data.test_type,
                "status": data.status,
                "url": url,
            })
        }
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn update_synthetics_test(
    ctx: ToolContext,
    public_id: SyntheticsTestId,
    name: Option<String>,
    url: Option<String>,
    locations: Option<Vec<String>>,
    message: Option<String>,
    tags: Option<Vec<String>>,
    tick_every: Option<i32>,
) -> anyhow::Result<Value> {
    let name = sanitize_optional(name, MAX_NAME_LENGTH);
    let message = sanitize_optional(message, MAX_MESSAGE_LENGTH);
    let tags = tags.map(sanitize_tags);

    info!("Updating Synthetics test: {}", public_id);

    let api = ctx.synthetics_api();

    // Get the existing test
    let existing = api.get_test(&public_id.0).await.map_err(|e| {
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
    let result = api.update_test(&public_id.0, &updated_request).await;

    tool_response_with_fields!(
        result,
        "synthetics_test_updated",
        ctx,
        data,
        format!("Updated Synthetics test: {}", data.name),
        {
            json!({
                "public_id": data.public_id,
                "name": data.name,
                "status": data.status,
            })
        }
    )
}

pub async fn trigger_synthetics_tests(
    ctx: ToolContext,
    test_ids: Vec<SyntheticsTestId>,
) -> anyhow::Result<Value> {
    info!("Triggering {} Synthetics test(s)", test_ids.len());

    if test_ids.is_empty() {
        return Ok(json!({
            "error": "At least one test ID must be provided",
            "status": "error",
        }));
    }

    let api = ctx.synthetics_api();
    let ids: Vec<String> = test_ids.iter().map(|id| id.0.clone()).collect();
    let result = api.trigger_tests(ids).await;

    tool_response_with_fields!(
        result,
        "synthetics_tests_triggered",
        ctx,
        data,
        {
            format!(
                "Triggered {} test(s), {} check(s) started",
                test_ids.len(),
                data.triggered_check_ids.len()
            )
        },
        {
            json!({
                "test_ids": test_ids,
                "triggered_check_ids": data.triggered_check_ids,
                "results": data.results,
            })
        }
    )
}
