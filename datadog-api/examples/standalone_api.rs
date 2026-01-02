//! Standalone API usage example.
//!
//! Shows how to use the datadog-api crate independently of the MCP server.
//!
//! Run with: cargo run --example standalone_api

use datadog_api::{
    apis::{DashboardsApi, MetricsApi, MonitorsApi, SyntheticsApi},
    config::DatadogConfig,
    DatadogClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = DatadogConfig::from_env()?;
    let client = DatadogClient::new(config)?;

    // The client can be cloned for use with multiple APIs
    let monitors_api = MonitorsApi::new(client.clone());
    let dashboards_api = DashboardsApi::new(client.clone());
    let metrics_api = MetricsApi::new(client.clone());
    let synthetics_api = SyntheticsApi::new(client);

    // Query monitors
    println!("=== Monitors ===");
    let monitors = monitors_api.list_monitors().await?;
    println!("Total monitors: {}", monitors.len());

    let alerting = monitors
        .iter()
        .filter(|m| m.overall_state.as_deref() == Some("Alert"))
        .count();
    println!("Currently alerting: {}", alerting);

    // Query dashboards
    println!("\n=== Dashboards ===");
    let dashboards = dashboards_api.list_dashboards().await?;
    let count = dashboards.dashboards.as_ref().map(|d| d.len()).unwrap_or(0);
    println!("Total dashboards: {}", count);

    // Query metrics
    println!("\n=== Metrics ===");
    let now = chrono::Utc::now().timestamp();
    let hour_ago = now - 3600;
    let metrics = metrics_api
        .query_metrics(hour_ago, now, "avg:system.cpu.user{*}")
        .await?;
    println!(
        "CPU metrics series: {}",
        metrics.series.as_ref().map(|s| s.len()).unwrap_or(0)
    );

    // Query synthetics
    println!("\n=== Synthetics ===");
    let tests = synthetics_api.list_tests().await?;
    let test_count = tests.tests.as_ref().map(|t| t.len()).unwrap_or(0);
    println!("Synthetic tests: {}", test_count);

    Ok(())
}
