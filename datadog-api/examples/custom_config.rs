//! Custom configuration example for the Datadog API client.
//!
//! Demonstrates different ways to configure the client.
//!
//! Run with: cargo run --example custom_config

use datadog_api::{apis::MetricsApi, config::DatadogConfig, DatadogClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Method 1: Direct configuration
    let config = DatadogConfig::new(
        "your-api-key".to_string(),
        "your-app-key".to_string(),
    )
    .with_site("datadoghq.eu".to_string()); // Use EU datacenter

    println!("Config 1 - Site: {}", config.site);
    println!("Config 1 - Base URL: {}", config.base_url());

    // Method 2: From environment variables
    // Requires DD_API_KEY, DD_APP_KEY, and optionally DD_SITE
    if let Ok(env_config) = DatadogConfig::from_env() {
        println!("\nConfig 2 (from env) - Site: {}", env_config.site);
    }

    // Method 3: From credentials file or environment
    // Tries ~/.datadog-mcp/credentials.json first, then keyring, then env vars
    if let Ok(file_config) = DatadogConfig::from_env_or_file() {
        println!("\nConfig 3 (from file/env) - Site: {}", file_config.site);

        // Create client and use it
        let client = DatadogClient::new(file_config)?;
        let metrics_api = MetricsApi::new(client);

        // List metrics matching a pattern
        let metrics = metrics_api.list_metrics("system.cpu").await?;
        println!(
            "Found {} metrics matching 'system.cpu'",
            metrics.metrics.as_ref().map(|m| m.len()).unwrap_or(0)
        );
    }

    Ok(())
}
