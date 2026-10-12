//! Slack integration tools

use crate::input_validation::{validate_slack_account_name, validate_slack_channel_name};
use crate::response::{simple_success_with_fields, CachePolicy, ToolOutput};
use crate::state::ToolContext;
use crate::tool_inputs::SlackChannelDisplayInput;
use datadog_api::models::{Monitor, SlackIntegrationChannel, SlackIntegrationChannelDisplay};
use regex::regex;
use serde::Serialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use tracing::info;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct MonitorReference {
    id: Option<i64>,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChannelNotificationAudit {
    channel_name: String,
    accepted_handles: Vec<String>,
    referencing_monitors: Vec<MonitorReference>,
}

#[derive(Debug, Serialize)]
struct SlackNotificationAudit {
    account_name: String,
    configured_channels: Vec<ChannelNotificationAudit>,
    unused_channels: Vec<String>,
    unmatched_handles_for_account: BTreeMap<String, Vec<MonitorReference>>,
    dynamic_handles: BTreeMap<String, Vec<MonitorReference>>,
}

pub async fn get_slack_channels(
    ctx: ToolContext,
    account_name: String,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    info!(account_name, "Getting Datadog Slack integration channels");

    let result = ctx.slack_api().list_channels(&account_name).await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        format!(
            "Retrieved {} Slack channel configuration(s) for account '{account_name}'",
            data.len()
        ),
        {
            json!({
                "account_name": account_name,
                "channel_count": data.len(),
            })
        }
    )
}

pub async fn get_slack_channel(
    ctx: ToolContext,
    account_name: String,
    channel_name: String,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    validate_slack_channel_name(&channel_name)?;
    info!(
        account_name,
        channel_name, "Getting Datadog Slack channel configuration"
    );

    let result = ctx
        .slack_api()
        .get_channel(&account_name, &channel_name)
        .await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        format!("Retrieved Slack channel configuration '{channel_name}'"),
        {
            json!({
                "account_name": account_name,
                "channel_name": data.name,
            })
        }
    )
}

pub async fn create_slack_channel(
    ctx: ToolContext,
    account_name: String,
    channel_name: String,
    display: Option<SlackChannelDisplayInput>,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    validate_slack_channel_name(&channel_name)?;
    validate_display_update(display.as_ref())?;
    info!(
        account_name,
        channel_name, "Creating Datadog Slack channel configuration"
    );

    let channel = SlackIntegrationChannel {
        display: display.map(api_display),
        name: Some(channel_name.clone()),
        ..Default::default()
    };
    let result = ctx
        .slack_api()
        .create_channel(&account_name, &channel)
        .await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        format!("Created Slack channel configuration '{channel_name}'"),
        {
            json!({
                "account_name": account_name,
                "channel_name": data.name,
                "operation": "created",
            })
        }
    )
}

pub async fn update_slack_channel(
    ctx: ToolContext,
    account_name: String,
    channel_name: String,
    new_channel_name: Option<String>,
    display: Option<SlackChannelDisplayInput>,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    validate_slack_channel_name(&channel_name)?;
    if let Some(name) = new_channel_name.as_deref() {
        validate_slack_channel_name(name)?;
    }
    validate_display_update(display.as_ref())?;
    if new_channel_name.is_none() && display.is_none() {
        anyhow::bail!("at least one of new_channel_name or display must be provided");
    }
    info!(
        account_name,
        channel_name, "Updating Datadog Slack channel configuration"
    );

    let channel = SlackIntegrationChannel {
        display: display.map(api_display),
        name: new_channel_name,
        ..Default::default()
    };
    let result = ctx
        .slack_api()
        .update_channel(&account_name, &channel_name, &channel)
        .await;
    tool_response_with_fields!(
        result,
        no_cache,
        data,
        format!("Updated Slack channel configuration '{channel_name}'"),
        {
            json!({
                "account_name": account_name,
                "previous_channel_name": channel_name,
                "channel_name": data.name,
                "operation": "updated",
            })
        }
    )
}

pub async fn delete_slack_channel(
    ctx: ToolContext,
    account_name: String,
    channel_name: String,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    validate_slack_channel_name(&channel_name)?;
    info!(
        account_name,
        channel_name, "Deleting Datadog Slack channel configuration"
    );

    ctx.slack_api()
        .delete_channel(&account_name, &channel_name)
        .await?;
    simple_success_with_fields(
        format!("Deleted Slack channel configuration '{channel_name}'"),
        json!({
            "account_name": account_name,
            "channel_name": channel_name,
            "operation": "deleted",
        }),
    )
}

pub async fn audit_slack_monitor_notifications(
    ctx: ToolContext,
    account_name: String,
) -> anyhow::Result<ToolOutput> {
    validate_slack_account_name(&account_name)?;
    info!(account_name, "Auditing monitor Slack notification handles");

    let slack_api = ctx.slack_api();
    let monitors_api = ctx.monitors_api();
    let (channels, monitors) = tokio::try_join!(
        slack_api.list_channels(&account_name),
        monitors_api.list_monitors(),
    )?;
    let audit = build_notification_audit(&account_name, &channels, &monitors);
    let referenced_channel_count = audit
        .configured_channels
        .iter()
        .filter(|channel| !channel.referencing_monitors.is_empty())
        .count();
    let unused_channel_count = audit.unused_channels.len();
    let unmatched_handle_count = audit.unmatched_handles_for_account.len();
    let dynamic_handle_count = audit.dynamic_handles.len();

    ToolOutput::from_data(
        &audit,
        format!(
            "Audited {} configured Slack channel(s): {referenced_channel_count} referenced, {unused_channel_count} unused, {unmatched_handle_count} unmatched handle(s)",
            channels.len()
        ),
        json!({
            "account_name": account_name,
            "configured_channel_count": channels.len(),
            "referenced_channel_count": referenced_channel_count,
            "unused_channel_count": unused_channel_count,
            "unmatched_handle_count": unmatched_handle_count,
            "dynamic_handle_count": dynamic_handle_count,
            "monitor_count": monitors.len(),
        }),
        CachePolicy::Never,
    )
}

fn validate_display_update(display: Option<&SlackChannelDisplayInput>) -> anyhow::Result<()> {
    if display.is_some_and(SlackChannelDisplayInput::is_empty) {
        anyhow::bail!("display must contain at least one setting when provided");
    }
    Ok(())
}

fn api_display(display: SlackChannelDisplayInput) -> SlackIntegrationChannelDisplay {
    SlackIntegrationChannelDisplay {
        message: display.message,
        mute_buttons: display.mute_buttons,
        notified: display.notified,
        snapshot: display.snapshot,
        tags: display.tags,
        ..Default::default()
    }
}

fn build_notification_audit(
    account_name: &str,
    channels: &[SlackIntegrationChannel],
    monitors: &[Monitor],
) -> SlackNotificationAudit {
    let configured = channels
        .iter()
        .filter_map(|channel| channel.name.as_deref())
        .map(|channel_name| {
            let normalized_channel = normalize_handle_component(channel_name);
            let normalized_account = normalize_handle_component(account_name);
            let handles = BTreeSet::from([
                format!("@slack-{normalized_channel}"),
                format!("@slack-{normalized_account}-{normalized_channel}"),
            ]);
            (channel_name.to_string(), handles)
        })
        .collect::<Vec<_>>();

    let mut references = vec![BTreeSet::new(); configured.len()];
    let mut unmatched = BTreeMap::<String, BTreeSet<MonitorReference>>::new();
    let mut dynamic = BTreeMap::<String, BTreeSet<MonitorReference>>::new();

    for monitor in monitors {
        let Some(message) = monitor.message.as_deref() else {
            continue;
        };
        let monitor_reference = MonitorReference {
            id: monitor.id,
            name: monitor.name.clone(),
        };
        for handle in slack_handles(message) {
            let normalized = handle.to_ascii_lowercase();
            if normalized.contains("{{") {
                dynamic
                    .entry(handle)
                    .or_default()
                    .insert(monitor_reference.clone());
                continue;
            }
            if let Some((index, _)) = configured
                .iter()
                .enumerate()
                .find(|(_, (_, handles))| handles.contains(&normalized))
            {
                references[index].insert(monitor_reference.clone());
            } else {
                unmatched
                    .entry(handle)
                    .or_default()
                    .insert(monitor_reference.clone());
            }
        }
    }

    let configured_channels = configured
        .into_iter()
        .zip(references)
        .map(
            |((channel_name, handles), references)| ChannelNotificationAudit {
                channel_name,
                accepted_handles: handles.into_iter().collect(),
                referencing_monitors: references.into_iter().collect(),
            },
        )
        .collect::<Vec<_>>();
    let unused_channels = configured_channels
        .iter()
        .filter(|channel| channel.referencing_monitors.is_empty())
        .map(|channel| channel.channel_name.clone())
        .collect();

    SlackNotificationAudit {
        account_name: account_name.to_string(),
        configured_channels,
        unused_channels,
        unmatched_handles_for_account: sets_to_vectors(unmatched),
        dynamic_handles: sets_to_vectors(dynamic),
    }
}

fn normalize_handle_component(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('#')
        .to_ascii_lowercase()
        .replace(char::is_whitespace, "-")
}

fn slack_handles(message: &str) -> BTreeSet<String> {
    regex!(r"(?i)@slack-[A-Za-z0-9_#.{}/-]+")
        .find_iter(message)
        .map(|found| found.as_str().to_string())
        .collect()
}

fn sets_to_vectors(
    values: BTreeMap<String, BTreeSet<MonitorReference>>,
) -> BTreeMap<String, Vec<MonitorReference>> {
    values
        .into_iter()
        .map(|(key, value)| (key, value.into_iter().collect()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_matches_direct_and_account_qualified_handles() {
        let channels = vec![SlackIntegrationChannel {
            name: Some("#database-alerts".to_string()),
            ..Default::default()
        }];
        let monitors = vec![Monitor {
            id: Some(42),
            name: Some("SQL connection".to_string()),
            message: Some(
                "Notify @slack-database-alerts and @slack-Spiris-database-alerts".to_string(),
            ),
            ..empty_monitor()
        }];

        let audit = build_notification_audit("Spiris", &channels, &monitors);

        assert_eq!(audit.configured_channels[0].referencing_monitors.len(), 1);
        assert!(audit.unused_channels.is_empty());
        assert!(audit.unmatched_handles_for_account.is_empty());
    }

    #[test]
    fn audit_separates_dynamic_and_unmatched_handles() {
        let monitors = vec![Monitor {
            id: Some(7),
            name: Some("Routed alert".to_string()),
            message: Some("@slack-{{owner.name}} @slack-missing".to_string()),
            ..empty_monitor()
        }];

        let audit = build_notification_audit("Spiris", &[], &monitors);

        assert!(audit.dynamic_handles.contains_key("@slack-{{owner.name}}"));
        assert!(audit
            .unmatched_handles_for_account
            .contains_key("@slack-missing"));
    }

    fn empty_monitor() -> Monitor {
        Monitor {
            id: None,
            name: None,
            monitor_type: None,
            query: None,
            message: None,
            tags: None,
            options: None,
            overall_state: None,
            overall_state_modified: None,
            state: None,
            matching_downtimes: None,
            created: None,
            modified: None,
            creator: None,
            additional_properties: Default::default(),
        }
    }
}
