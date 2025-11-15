use crate::{client::DatadogClient, models::EventsResponse, Result};
use serde::Serialize;

pub struct EventsApi {
    client: DatadogClient,
}

impl EventsApi {
    pub fn new(client: DatadogClient) -> Self {
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
