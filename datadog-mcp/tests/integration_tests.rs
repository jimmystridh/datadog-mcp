use datadog_api::{apis::*, config::DatadogConfig, models::*, DatadogClient};
use serde_json::json;

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_creation() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

    assert_eq!(config.api_key, "test_api_key");
    assert_eq!(config.app_key, "test_app_key");
    assert_eq!(config.site, "datadoghq.com");
    assert_eq!(config.retry_config.max_retries, 3);
}

#[test]
fn test_config_with_site() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
        .with_site("datadoghq.eu".to_string());

    assert_eq!(config.site, "datadoghq.eu");
    assert_eq!(config.base_url(), "https://api.datadoghq.eu");
}

#[test]
fn test_unstable_operations_default() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

    assert!(config
        .unstable_operations
        .contains(&"incidents".to_string()));
}

// ============================================================================
// Model Serialization Tests
// ============================================================================

#[test]
fn test_synthetics_test_type_serialization() {
    let test_type = SyntheticsTestType::Api;
    let json = serde_json::to_string(&test_type).unwrap();
    assert_eq!(json, "\"api\"");

    let deserialized: SyntheticsTestType = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized, SyntheticsTestType::Api));
}

#[test]
fn test_synthetics_assertion_serialization() {
    let assertion = SyntheticsAssertion {
        assertion_type: SyntheticsAssertionType::StatusCode,
        operator: SyntheticsAssertionOperator::Is,
        target: json!(200),
    };

    let json = serde_json::to_string(&assertion).unwrap();
    assert!(json.contains("\"type\":\"statusCode\""));
    assert!(json.contains("\"operator\":\"is\""));
    assert!(json.contains("\"target\":200"));
}

#[test]
fn test_synthetics_test_request_deserialization() {
    let json = r#"{
        "method": "GET",
        "url": "https://api.example.com",
        "timeout": 30.0,
        "headers": {"Authorization": "Bearer token"}
    }"#;

    let request: SyntheticsTestRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.method, "GET");
    assert_eq!(request.url, "https://api.example.com");
    assert_eq!(request.timeout, Some(30.0));
    assert!(request.headers.is_some());
}

#[test]
fn test_synthetics_test_create_request_serialization() {
    let request = SyntheticsTestCreateRequest {
        name: "Test API Health".to_string(),
        test_type: SyntheticsTestType::Api,
        subtype: SyntheticsTestSubtype::Http,
        config: SyntheticsTestConfig {
            request: SyntheticsTestRequest {
                method: "GET".to_string(),
                url: "https://api.example.com/health".to_string(),
                timeout: Some(30.0),
                headers: None,
                body: None,
            },
            assertions: vec![SyntheticsAssertion {
                assertion_type: SyntheticsAssertionType::StatusCode,
                operator: SyntheticsAssertionOperator::Is,
                target: json!(200),
            }],
        },
        options: SyntheticsTestOptions {
            tick_every: 300,
            retry: None,
            min_failure_duration: None,
            min_location_failed: None,
        },
        locations: vec!["aws:eu-central-1".to_string()],
        message: Some("API health check failed".to_string()),
        tags: Some(vec!["env:prod".to_string()]),
        status: None,
    };

    // Test that it serializes and deserializes correctly
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: SyntheticsTestCreateRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, "Test API Health");
    assert_eq!(deserialized.locations.len(), 1);
    assert_eq!(deserialized.locations[0], "aws:eu-central-1");
}

// ============================================================================
// Tool Input Tests
// ============================================================================

#[test]
fn test_create_synthetics_test_input_schema() {
    use datadog_mcp::tool_inputs::CreateSyntheticsTestInput;
    use schemars::schema_for;

    let schema = schema_for!(CreateSyntheticsTestInput);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();

    assert!(schema_json.contains("name"));
    assert!(schema_json.contains("url"));
    assert!(schema_json.contains("locations"));
}

#[test]
fn test_update_synthetics_test_input_optional_fields() {
    use datadog_mcp::tool_inputs::UpdateSyntheticsTestInput;

    let input = UpdateSyntheticsTestInput {
        public_id: "abc-123".to_string(),
        name: Some("Updated Name".to_string()),
        url: None,
        locations: None,
        message: None,
        tags: None,
        tick_every: None,
    };

    assert_eq!(input.public_id, "abc-123");
    assert_eq!(input.name, Some("Updated Name".to_string()));
    assert!(input.url.is_none());
}

#[test]
fn test_get_kubernetes_deployments_input() {
    use datadog_mcp::tool_inputs::GetKubernetesDeploymentsInput;

    let input_all = GetKubernetesDeploymentsInput { namespace: None };
    assert!(input_all.namespace.is_none());

    let input_filtered = GetKubernetesDeploymentsInput {
        namespace: Some("production".to_string()),
    };
    assert_eq!(input_filtered.namespace, Some("production".to_string()));
}

// ============================================================================
// API Method Tests (without actual HTTP calls)
// ============================================================================

#[test]
fn test_synthetics_api_creation() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());
    let client = DatadogClient::new(config).unwrap();
    let _api = SyntheticsApi::new(client);
    // Just verify we can create the API instance
}

#[test]
fn test_downtimes_api_creation() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());
    let client = DatadogClient::new(config).unwrap();
    let _api = DowntimesApi::new(client);
    // Just verify we can create the API instance
}

#[test]
fn test_metrics_api_creation() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());
    let client = DatadogClient::new(config).unwrap();
    let _api = MetricsApi::new(client);
    // Just verify we can create the API instance
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_config_from_env_missing_keys() {
    std::env::remove_var("DD_API_KEY");
    std::env::remove_var("DD_APP_KEY");

    let result = DatadogConfig::from_env();
    assert!(result.is_err());
}

#[test]
fn test_retry_config_defaults() {
    use datadog_api::config::RetryConfig;

    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_backoff_ms, 100);
    assert_eq!(config.max_backoff_ms, 10000);
    assert_eq!(config.backoff_multiplier, 2.0);
}

// ============================================================================
// Cache Tests
// ============================================================================

#[tokio::test]
async fn test_cache_store_and_load() {
    use datadog_mcp::cache::{init_cache_in, load_data, store_data_in};
    use datadog_mcp::output::OutputFormat;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    init_cache_in(temp_dir.path()).await.unwrap();

    let test_data = json!({
        "test": "value",
        "number": 42,
        "array": [1, 2, 3]
    });

    let filepath = store_data_in(&test_data, "test", OutputFormat::Json, temp_dir.path())
        .await
        .unwrap();
    assert!(filepath.contains("test_"));
    assert!(filepath.ends_with(".json"));

    let loaded = load_data(&filepath).await.unwrap();
    assert_eq!(loaded, test_data);
}

#[tokio::test]
async fn test_cache_unique_filenames() {
    use datadog_mcp::cache::{init_cache_in, store_data_in};
    use datadog_mcp::output::OutputFormat;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    init_cache_in(temp_dir.path()).await.unwrap();

    let data1 = json!({"id": 1});
    let data2 = json!({"id": 2});

    let filepath1 = store_data_in(&data1, "unique", OutputFormat::Json, temp_dir.path())
        .await
        .unwrap();
    let filepath2 = store_data_in(&data2, "unique", OutputFormat::Json, temp_dir.path())
        .await
        .unwrap();

    assert_ne!(filepath1, filepath2);
}
