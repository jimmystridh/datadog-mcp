pub mod cache;
mod output;
#[macro_use]
pub mod response;
mod server;
mod state;
mod tool_inputs;
mod tools;
mod tools_part2;

use anyhow::Result;
use clap::Parser;
use datadog_api::DatadogConfig;
use output::OutputFormat;
use rmcp::{transport::stdio, ServiceExt};
use server::DatadogMcpServer;
use state::ServerState;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "datadog-mcp")]
#[command(about = "Datadog MCP Server - Model Context Protocol server for Datadog API", long_about = None)]
struct Args {
    /// Output format for MCP responses (json or toon)
    #[arg(long, default_value = "toon", value_parser = parse_format)]
    format: OutputFormat,
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "json" => Ok(OutputFormat::Json),
        "toon" => Ok(OutputFormat::Toon),
        _ => Err(format!("Invalid format '{}'. Must be 'json' or 'toon'", s)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Load environment variables from .env file and force override
    if let Ok(path) = dotenv::dotenv() {
        // Parse .env file and explicitly set environment variables to override any existing ones
        for line in std::fs::read_to_string(&path)?.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                std::env::set_var(key.trim(), value.trim());
            }
        }
    }

    // Initialize tracing to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting Datadog MCP Server");
    info!("Output format: {:?}", args.format);

    // Initialize cache
    cache::init_cache().await?;

    // Load Datadog configuration
    let config = DatadogConfig::from_env()?;
    info!("Loaded Datadog configuration for site: {}", config.site);

    // Initialize server state with output format
    let state = ServerState::new(config, args.format).await?;
    info!("Server state initialized");

    // Test connection to Datadog
    match state.test_connection().await {
        Ok(_) => info!(
            "Successfully connected to Datadog at: {}",
            state.config.site
        ),
        Err(e) => info!(
            "Could not verify connection to Datadog: {}. Server will start anyway, tools may fail.",
            e
        ),
    }

    // Create MCP server with our Datadog tools
    let server = DatadogMcpServer::new(state);
    info!("MCP server created with tools registered");

    // Start the server with stdio transport
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    info!("MCP server running, waiting for requests");
    service.waiting().await?;

    Ok(())
}
