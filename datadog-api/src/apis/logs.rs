use crate::{
    client::DatadogClient,
    models::{LogsSearchRequest, LogsSearchResponse},
    Result,
};

pub struct LogsApi {
    client: DatadogClient,
}

impl LogsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn search_logs(&self, request: &LogsSearchRequest) -> Result<LogsSearchResponse> {
        self.client
            .post("/api/v2/logs/events/search", request)
            .await
    }
}
