use crate::{
    client::DatadogClient,
    models::{LogsSearchRequest, LogsSearchResponse},
    Result,
};

/// API client for Datadog logs endpoints.
pub struct LogsApi {
    client: DatadogClient,
}

impl LogsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn search_logs(&self, request: &LogsSearchRequest) -> Result<LogsSearchResponse> {
        self.client
            .post("/api/v2/logs/events/search", request)
            .await
    }
}
