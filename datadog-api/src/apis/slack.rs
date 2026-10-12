use crate::{client::DatadogClient, models::SlackIntegrationChannel, Result};
use std::fmt::Write;

/// API client for Datadog Slack integration channel configuration.
pub struct SlackApi {
    client: DatadogClient,
}

impl SlackApi {
    /// Creates a new Slack integration API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_channels(&self, account_name: &str) -> Result<Vec<SlackIntegrationChannel>> {
        self.client.get(&channels_endpoint(account_name)).await
    }

    pub async fn get_channel(
        &self,
        account_name: &str,
        channel_name: &str,
    ) -> Result<SlackIntegrationChannel> {
        self.client
            .get(&channel_endpoint(account_name, channel_name))
            .await
    }

    pub async fn create_channel(
        &self,
        account_name: &str,
        channel: &SlackIntegrationChannel,
    ) -> Result<SlackIntegrationChannel> {
        self.client
            .post(&channels_endpoint(account_name), channel)
            .await
    }

    pub async fn update_channel(
        &self,
        account_name: &str,
        channel_name: &str,
        channel: &SlackIntegrationChannel,
    ) -> Result<SlackIntegrationChannel> {
        self.client
            .patch(&channel_endpoint(account_name, channel_name), channel)
            .await
    }

    pub async fn delete_channel(&self, account_name: &str, channel_name: &str) -> Result<()> {
        self.client
            .delete(&channel_endpoint(account_name, channel_name))
            .await
    }
}

fn channels_endpoint(account_name: &str) -> String {
    format!(
        "/api/v1/integration/slack/configuration/accounts/{}/channels",
        encode_path_segment(account_name)
    )
}

fn channel_endpoint(account_name: &str, channel_name: &str) -> String {
    format!(
        "{}/{}",
        channels_endpoint(account_name),
        encode_path_segment(channel_name)
    )
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(&mut encoded, "%{byte:02X}").expect("writing to a String cannot fail");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_segments_are_percent_encoded() {
        assert_eq!(
            channel_endpoint("main account", "#ops/alerts"),
            "/api/v1/integration/slack/configuration/accounts/main%20account/channels/%23ops%2Falerts"
        );
    }
}
