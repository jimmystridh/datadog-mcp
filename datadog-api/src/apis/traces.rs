use crate::{
    client::DatadogClient,
    models::{
        ApmServiceDependenciesResponse, ServiceListResponse, SpansSearchRequest,
        SpansSearchResponse,
    },
    Result,
};
use serde::Serialize;

/// API client for Datadog APM span search and service discovery endpoints.
pub struct TracesApi {
    client: DatadogClient,
}

impl TracesApi {
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    /// Search indexed spans using the public v2 Spans API.
    pub async fn search_spans(&self, request: &SpansSearchRequest) -> Result<SpansSearchResponse> {
        self.client
            .post_retryable("/api/v2/spans/events/search", request)
            .await
    }

    /// List APM services for an environment. Use `*` for all environments.
    pub async fn list_services(&self, environment: &str) -> Result<ServiceListResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            #[serde(rename = "filter[env]")]
            environment: &'a str,
        }

        self.client
            .get_with_query("/api/v2/apm/services", &QueryParams { environment })
            .await
    }

    /// Get all APM service dependencies for an environment and time range.
    pub async fn get_service_dependencies(
        &self,
        environment: &str,
        primary_tag: Option<&str>,
        start: Option<i64>,
        end: Option<i64>,
    ) -> Result<ApmServiceDependenciesResponse> {
        #[derive(Serialize)]
        struct QueryParams<'a> {
            env: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            primary_tag: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            end: Option<i64>,
        }

        self.client
            .get_with_query(
                "/api/v1/service_dependencies",
                &QueryParams {
                    env: environment,
                    primary_tag,
                    start,
                    end,
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{
            SpansSearchData, SpansSearchFilter, SpansSearchPage, SpansSearchRequestAttributes,
        },
        DatadogConfig,
    };

    #[test]
    fn test_traces_api_creation() {
        let config = DatadogConfig::new("test_key".to_string(), "test_app_key".to_string());
        let client = DatadogClient::new(config).unwrap();
        let _traces_api = TracesApi::new(client);
    }

    #[test]
    fn test_span_search_request_serialization() {
        let request = SpansSearchRequest {
            data: SpansSearchData {
                attributes: SpansSearchRequestAttributes {
                    filter: SpansSearchFilter {
                        from: "now-15m".to_string(),
                        query: "service:web".to_string(),
                        to: "now".to_string(),
                    },
                    page: Some(SpansSearchPage {
                        cursor: None,
                        limit: Some(25),
                    }),
                    sort: Some("timestamp".to_string()),
                },
                resource_type: "search_request".to_string(),
            },
        };

        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["data"]["type"], "search_request");
        assert_eq!(json["data"]["attributes"]["page"]["limit"], 25);
    }
}
