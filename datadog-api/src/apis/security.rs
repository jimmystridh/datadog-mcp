use crate::{client::DatadogClient, models::SecurityRulesResponse, Result};

/// API client for Datadog security endpoints.
pub struct SecurityApi {
    client: DatadogClient,
}

impl SecurityApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_security_rules(&self) -> Result<SecurityRulesResponse> {
        self.client
            .get("/api/v2/security_monitoring/rules")
            .await
    }
}
