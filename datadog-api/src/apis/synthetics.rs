use crate::{
    client::DatadogClient,
    models::{
        SyntheticsDeleteTestsRequest, SyntheticsDeleteTestsResponse, SyntheticsLocationsResponse,
        SyntheticsTestCreateRequest, SyntheticsTestResponse, SyntheticsTestsResponse,
        SyntheticsTriggerRequest, SyntheticsTriggerResponse, SyntheticsTriggerTest,
    },
    Result,
};

/// API client for Datadog synthetics endpoints.
pub struct SyntheticsApi {
    client: DatadogClient,
}

impl SyntheticsApi {
    /// Creates a new API client.
    #[must_use]
    pub const fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn list_tests(&self) -> Result<SyntheticsTestsResponse> {
        self.client.get("/api/v1/synthetics/tests").await
    }

    pub async fn get_test(&self, public_id: &str) -> Result<SyntheticsTestResponse> {
        let endpoint = format!("/api/v1/synthetics/tests/{}", public_id);
        self.client.get(&endpoint).await
    }

    pub async fn list_locations(&self) -> Result<SyntheticsLocationsResponse> {
        self.client.get("/api/v1/synthetics/locations").await
    }

    pub async fn create_test(
        &self,
        request: &SyntheticsTestCreateRequest,
    ) -> Result<SyntheticsTestResponse> {
        self.client
            .post("/api/v1/synthetics/tests/api", request)
            .await
    }

    pub async fn update_test(
        &self,
        public_id: &str,
        request: &SyntheticsTestCreateRequest,
    ) -> Result<SyntheticsTestResponse> {
        let endpoint = format!("/api/v1/synthetics/tests/api/{}", public_id);
        self.client.put(&endpoint, request).await
    }

    pub async fn trigger_tests(&self, test_ids: Vec<String>) -> Result<SyntheticsTriggerResponse> {
        let request = SyntheticsTriggerRequest {
            tests: test_ids
                .into_iter()
                .map(|id| SyntheticsTriggerTest { public_id: id })
                .collect(),
        };

        self.client
            .post("/api/v1/synthetics/tests/trigger", &request)
            .await
    }

    pub async fn delete_tests(
        &self,
        request: &SyntheticsDeleteTestsRequest,
    ) -> Result<SyntheticsDeleteTestsResponse> {
        self.client
            .post("/api/v1/synthetics/tests/delete", request)
            .await
    }
}
