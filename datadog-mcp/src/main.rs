//! Datadog MCP Server
//!
//! A Model Context Protocol server that exposes Datadog tools to AI assistants.
//! Runs over stdio and provides access to monitors, dashboards, metrics, logs,
//! synthetics, and more through the MCP protocol.

pub mod cache;
mod errors;
mod ids;
mod input_validation;
mod output;
#[macro_use]
pub mod response;
mod sanitize;
mod server;
mod state;
mod tool_inputs;
mod tools;

use anyhow::Result;
use clap::Parser;
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
    #[cfg_attr(feature = "toon", arg(long, default_value = "toon", value_parser = parse_format))]
    #[cfg_attr(not(feature = "toon"), arg(long, default_value = "json", value_parser = parse_format))]
    format: OutputFormat,

    /// Store credentials from env or file into the system keyring instead of starting the server
    #[cfg(feature = "keyring")]
    #[arg(long)]
    store_credentials: bool,
}

fn parse_format(s: &str) -> Result<OutputFormat, String> {
    match s.to_lowercase().as_str() {
        "json" => Ok(OutputFormat::Json),
        #[cfg(feature = "toon")]
        "toon" => Ok(OutputFormat::Toon),
        #[cfg(feature = "toon")]
        _ => Err(format!("Invalid format '{}'. Must be 'json' or 'toon'", s)),
        #[cfg(not(feature = "toon"))]
        _ => Err(format!("Invalid format '{}'. Must be 'json'", s)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Load environment variables from .env file without overriding existing variables
    let _ = dotenv::dotenv();

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
    let config = datadog_api::config::DatadogConfig::from_env_or_file()?;

    // If requested, store credentials in keyring and exit
    #[cfg(feature = "keyring")]
    if args.store_credentials {
        config.store_in_keyring()?;
        info!("Stored Datadog credentials in keyring");
        return Ok(());
    }
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
