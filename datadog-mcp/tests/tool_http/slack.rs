use super::*;
use datadog_mcp::tool_inputs::SlackChannelDisplayInput;

#[tokio::test]
async fn slack_channel_tools_manage_display_configuration() {
    let server = MockServer::start().await;
    let channels_endpoint = "/api/v1/integration/slack/configuration/accounts/Spiris/channels";
    let channel_endpoint = format!("{channels_endpoint}/%23database-alerts");
    let full_channel = json!({
        "name": "#database-alerts",
        "display": {
            "message": true,
            "mute_buttons": false,
            "notified": true,
            "snapshot": true,
            "tags": true
        }
    });
    Mock::given(method("GET"))
        .and(path(channels_endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([full_channel.clone()])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(&channel_endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(full_channel.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(channels_endpoint))
        .and(body_json(full_channel.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(full_channel.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path(&channel_endpoint))
        .and(body_json(json!({
            "display":{"mute_buttons":true},
            "name":"#database-warnings"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name":"#database-warnings",
            "display":{"mute_buttons":true}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(&channel_endpoint))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let listed = call_tool!(
        ctx,
        tools::get_slack_channels(ctx.clone(), "Spiris".to_string())
    );
    assert_success(&listed);
    assert_eq!(listed["channel_count"], 1);

    let selected = call_tool!(
        ctx,
        tools::get_slack_channel(
            ctx.clone(),
            "Spiris".to_string(),
            "#database-alerts".to_string(),
        )
    );
    assert_success(&selected);
    assert_eq!(selected["data"]["display"]["snapshot"], true);

    let created = call_tool!(
        ctx,
        tools::create_slack_channel(
            ctx.clone(),
            "Spiris".to_string(),
            "#database-alerts".to_string(),
            Some(SlackChannelDisplayInput {
                message: Some(true),
                mute_buttons: Some(false),
                notified: Some(true),
                snapshot: Some(true),
                tags: Some(true),
            }),
        )
    );
    assert_success(&created);
    assert_eq!(created["operation"], "created");

    let updated = call_tool!(
        ctx,
        tools::update_slack_channel(
            ctx.clone(),
            "Spiris".to_string(),
            "#database-alerts".to_string(),
            Some("#database-warnings".to_string()),
            Some(SlackChannelDisplayInput {
                mute_buttons: Some(true),
                ..Default::default()
            }),
        )
    );
    assert_success(&updated);
    assert_eq!(updated["data"]["display"]["mute_buttons"], true);
    assert_eq!(updated["channel_name"], "#database-warnings");

    let deleted = call_tool!(
        ctx,
        tools::delete_slack_channel(
            ctx.clone(),
            "Spiris".to_string(),
            "#database-alerts".to_string(),
        )
    );
    assert_success(&deleted);
    assert_eq!(deleted["operation"], "deleted");
}

#[tokio::test]
async fn slack_notification_audit_correlates_monitor_handles() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/integration/slack/configuration/accounts/Spiris/channels",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"name":"#database-alerts"},
            {"name":"#unused-alerts"}
        ])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/monitor"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "id":42,
                "name":"SQL connection",
                "type":"service check",
                "message":"Notify @slack-Spiris-database-alerts"
            },
            {
                "id":43,
                "name":"Dynamic routing",
                "type":"metric alert",
                "message":"Notify @slack-{{owner.name}} and @slack-missing"
            }
        ])))
        .expect(1)
        .mount(&server)
        .await;

    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::audit_slack_monitor_notifications(ctx.clone(), "Spiris".to_string())
    );

    assert_success(&out);
    assert_eq!(out["configured_channel_count"], 2);
    assert_eq!(out["referenced_channel_count"], 1);
    assert_eq!(out["unused_channel_count"], 1);
    assert_eq!(out["unmatched_handle_count"], 1);
    assert_eq!(out["dynamic_handle_count"], 1);
    assert_eq!(out["data"]["unused_channels"][0], "#unused-alerts");
}

#[tokio::test]
async fn slack_updates_require_at_least_one_change() {
    let server = MockServer::start().await;
    let ctx = mock_context(&server).await;
    let out = call_tool!(
        ctx,
        tools::update_slack_channel(
            ctx.clone(),
            "Spiris".to_string(),
            "#database-alerts".to_string(),
            None,
            None,
        )
    );

    assert_eq!(out["status"], "error");
    assert!(out["error"].as_str().unwrap().contains("at least one"));
}
