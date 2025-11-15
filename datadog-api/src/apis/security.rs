use crate::{client::DatadogClient, models::SecurityRulesResponse, Result};

pub struct SecurityApi {
    client: DatadogClient,
}

impl SecurityApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_security_rules(&self) -> Result<SecurityRulesResponse> {
        self.client
            .get("/api/v2/security_monitoring/rules")
            .await
    }
}
