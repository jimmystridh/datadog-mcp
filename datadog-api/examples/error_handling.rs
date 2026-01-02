//! Error handling example for the Datadog API client.
//!
//! Demonstrates how to handle different error types from the API.
//!
//! Run with: cargo run --example error_handling

use datadog_api::{apis::MonitorsApi, config::DatadogConfig, DatadogClient, Error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;
    let monitors_api = MonitorsApi::new(client);

    // Try to get a monitor that doesn't exist
    match monitors_api.get_monitor(999999999).await {
        Ok(monitor) => {
            println!("Found monitor: {:?}", monitor.name);
        }
        Err(e) => {
            handle_error(&e);
        }
    }

    Ok(())
}

fn handle_error(error: &Error) {
    // Use the helper methods to identify error types
    if error.is_not_found() {
        println!("Resource not found");
    } else if error.is_unauthorized() {
        println!("Authentication failed");
        println!("Check your DD_API_KEY and DD_APP_KEY environment variables");
    } else if error.is_rate_limited() {
        println!("Rate limited - consider implementing exponential backoff");
    } else if error.is_retryable() {
        println!("Transient error - retry might succeed");
    } else {
        // Match on the error variant for detailed handling
        match error {
            Error::ApiError { status, message } => {
                println!("API error (status {}): {}", status, message);
            }
            Error::HttpError(e) => {
                println!("HTTP error: {}", e);
            }
            Error::ConfigError(msg) => {
                println!("Configuration error: {}", msg);
            }
            Error::JsonError(e) => {
                println!("JSON parsing error: {}", e);
            }
            _ => {
                println!("Unexpected error: {}", error);
            }
        }
    }
}
