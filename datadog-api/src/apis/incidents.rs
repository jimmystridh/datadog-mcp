use crate::{client::DatadogClient, models::IncidentsResponse, Result};
use serde::Serialize;

/// API client for Datadog incidents endpoints.
pub struct IncidentsApi {
    client: DatadogClient,
}

impl IncidentsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_incidents(
        &self,
        page_size: Option<i32>,
        page_offset: Option<i64>,
    ) -> Result<IncidentsResponse> {
        #[derive(Serialize)]
        struct QueryParams {
            #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
            page_size: Option<i32>,
            #[serde(rename = "page[offset]", skip_serializing_if = "Option::is_none")]
            page_offset: Option<i64>,
        }

        self.client
            .get_with_query(
                "/api/v2/incidents",
                &QueryParams {
                    page_size,
                    page_offset,
                },
            )
            .await
    }
}
