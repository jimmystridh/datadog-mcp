//! Basic usage example for the Datadog API client.
//!
//! Run with: cargo run --example basic_usage
//!
//! Requires environment variables:
//! - DD_API_KEY: Your Datadog API key
//! - DD_APP_KEY: Your Datadog application key
//! - DD_SITE (optional): Datadog site (default: datadoghq.com)

use datadog_api::{apis::MonitorsApi, config::DatadogConfig, DatadogClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment variables
    let config = DatadogConfig::from_env()?;
    println!("Connected to Datadog site: {}", config.site);

    // Create the client
    let client = DatadogClient::new(config)?;

    // Use the Monitors API
    let monitors_api = MonitorsApi::new(client);

    // List all monitors
    let monitors = monitors_api.list_monitors().await?;
    println!("Found {} monitors", monitors.len());

    // Print first 5 monitors
    for monitor in monitors.iter().take(5) {
        println!(
            "  - {} (ID: {:?}, State: {:?})",
            monitor.name.as_deref().unwrap_or("Unnamed"),
            monitor.id,
            monitor.overall_state
        );
    }

    Ok(())
}
