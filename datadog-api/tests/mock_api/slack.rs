use super::*;
use serde_json::json;

fn channel() -> SlackIntegrationChannel {
    SlackIntegrationChannel {
        name: Some("#database-alerts".to_string()),
        display: Some(SlackIntegrationChannelDisplay {
            message: Some(true),
            mute_buttons: Some(false),
            notified: Some(true),
            snapshot: Some(true),
            tags: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[tokio::test]
async fn slack_api_lists_and_gets_channels() {
    let server = MockServer::start().await;
    let response = channel();
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/integration/slack/configuration/accounts/Spiris/channels",
        ))
        .and(header("dd-api-key", "test_api_key"))
        .and(header("dd-application-key", "test_app_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![response.clone()]))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/api/v1/integration/slack/configuration/accounts/Spiris/channels/%23database-alerts",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .expect(1)
        .mount(&server)
        .await;

    let api = SlackApi::new(create_test_client(&server).await);
    let channels = api.list_channels("Spiris").await.unwrap();
    let selected = api.get_channel("Spiris", "#database-alerts").await.unwrap();

    assert_eq!(channels.len(), 1);
    assert_eq!(selected.name.as_deref(), Some("#database-alerts"));
    assert_eq!(selected.display.unwrap().snapshot, Some(true));
}

#[tokio::test]
async fn slack_api_creates_updates_and_deletes_channels() {
    let server = MockServer::start().await;
    let request = channel();
    let endpoint = "/api/v1/integration/slack/configuration/accounts/Spiris/channels";
    let selected_endpoint = format!("{endpoint}/%23database-alerts");
    Mock::given(method("POST"))
        .and(path(endpoint))
        .and(body_json(serde_json::to_value(&request).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(request.clone()))
        .expect(1)
        .mount(&server)
        .await;

    let update = SlackIntegrationChannel {
        display: Some(SlackIntegrationChannelDisplay {
            mute_buttons: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };
    Mock::given(method("PATCH"))
        .and(path(&selected_endpoint))
        .and(body_json(serde_json::to_value(&update).unwrap()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "#database-alerts",
            "display": {"mute_buttons": true}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path(&selected_endpoint))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let api = SlackApi::new(create_test_client(&server).await);
    api.create_channel("Spiris", &request).await.unwrap();
    let updated = api
        .update_channel("Spiris", "#database-alerts", &update)
        .await
        .unwrap();
    api.delete_channel("Spiris", "#database-alerts")
        .await
        .unwrap();

    assert_eq!(updated.display.unwrap().mute_buttons, Some(true));
}
