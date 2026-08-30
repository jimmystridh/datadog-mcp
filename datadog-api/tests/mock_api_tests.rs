//! Comprehensive mock tests for all Datadog API endpoints.

use datadog_api::models::*;
use datadog_api::{
    apis::*, DashboardDocument, DatadogClient, DatadogConfig, NumberedPage, OffsetPage,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn create_test_client(server: &MockServer) -> DatadogClient {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
        .with_base_url(server.uri());
    DatadogClient::new(config).unwrap()
}

#[path = "mock_api/platform.rs"]
mod platform;
#[path = "mock_api/resilience.rs"]
mod resilience;
#[path = "mock_api/resources.rs"]
mod resources;
#[path = "mock_api/telemetry.rs"]
mod telemetry;
