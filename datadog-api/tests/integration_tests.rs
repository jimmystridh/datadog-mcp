use datadog_api::{DatadogClient, DatadogConfig};

#[test]
fn test_client_creation() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

    let client = DatadogClient::new(config.clone());
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.config().api_key, "test_api_key");
    assert_eq!(client.config().app_key, "test_app_key");
}

#[test]
fn test_client_config_access() {
    let config = DatadogConfig::new("api_key".to_string(), "app_key".to_string())
        .with_site("datadoghq.eu".to_string());

    let client = DatadogClient::new(config).unwrap();
    assert_eq!(client.config().site, "datadoghq.eu");
}

#[test]
fn test_client_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<DatadogClient>();
}

#[test]
fn test_client_is_cloneable() {
    let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

    let client = DatadogClient::new(config).unwrap();
    let _cloned = client.clone();
}
