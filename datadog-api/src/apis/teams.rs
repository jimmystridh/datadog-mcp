use crate::{client::DatadogClient, models::TeamsResponse, Result};
use serde::Serialize;

/// API client for Datadog teams endpoints.
pub struct TeamsApi {
    client: DatadogClient,
}

impl TeamsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_teams(
        &self,
        page_number: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<TeamsResponse> {
        #[derive(Serialize)]
        struct QueryParams {
            #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
            page_number: Option<i64>,
            #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
            page_size: Option<i64>,
        }

        self.client
            .get_with_query(
                "/api/v2/team",
                &QueryParams {
                    page_number,
                    page_size,
                },
            )
            .await
    }
}
