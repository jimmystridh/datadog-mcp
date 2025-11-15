use crate::{client::DatadogClient, models::IncidentsResponse, Result};
use serde::Serialize;

pub struct IncidentsApi {
    client: DatadogClient,
}

impl IncidentsApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_incidents(&self, page_size: Option<i32>) -> Result<IncidentsResponse> {
        if let Some(size) = page_size {
            #[derive(Serialize)]
            struct QueryParams {
                #[serde(rename = "page[size]")]
                page_size: i32,
            }

            let params = QueryParams { page_size: size };

            self.client
                .get_with_query("/api/v2/incidents", &params)
                .await
        } else {
            self.client.get("/api/v2/incidents").await
        }
    }

    // Note: Full pagination support would require implementing async iteration
    // This is a simplified version
    pub async fn list_all_incidents(&self, page_size: i32) -> Result<Vec<crate::models::Incident>> {
        let mut all_incidents = Vec::new();
        let mut offset = 0;

        loop {
            #[derive(Serialize)]
            struct QueryParams {
                #[serde(rename = "page[size]")]
                page_size: i32,
                #[serde(rename = "page[offset]")]
                page_offset: i64,
            }

            let params = QueryParams {
                page_size,
                page_offset: offset,
            };

            let response: IncidentsResponse = self
                .client
                .get_with_query("/api/v2/incidents", &params)
                .await?;

            if let Some(incidents) = response.data {
                if incidents.is_empty() {
                    break;
                }
                all_incidents.extend(incidents);
            } else {
                break;
            }

            // Check if there are more pages
            if let Some(meta) = response.meta {
                if let Some(pagination) = meta.pagination {
                    if let Some(next_offset) = pagination.next_offset {
                        offset = next_offset;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(all_incidents)
    }
}
