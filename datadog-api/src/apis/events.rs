use crate::{client::DatadogClient, models::EventsResponse, Result};
use serde::Serialize;

/// API client for Datadog events endpoints.
pub struct EventsApi {
    client: DatadogClient,
}

impl EventsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_events(
        &self,
        start: i64,
        end: i64,
        priority: Option<&str>,
        sources: Option<&str>,
    ) -> Result<EventsResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            start: i64,
            end: i64,
            #[serde(skip_serializing_if = "Option::is_none")]
            priority: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            sources: Option<&'a str>,
        }

        let params = QueryParams {
            start,
            end,
            priority,
            sources,
        };

        self.client.get_with_query("/api/v1/events", &params).await
    }
}
