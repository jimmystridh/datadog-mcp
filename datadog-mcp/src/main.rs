//! Datadog MCP Server
//!
//! A Model Context Protocol server that exposes Datadog tools to AI assistants.
//! Runs over stdio and provides access to monitors, dashboards, metrics, logs,
//! synthetics, and more through the MCP protocol.

use anyhow::Result;
use clap::Parser;
use datadog_mcp::{
    cache,
    output::OutputFormat,
    server::DatadogMcpServer,
    state::{ServerOptions, ServerState},
};
use rmcp::{transport::stdio, ServiceExt};
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

    /// Allow tools that create, update, delete, trigger, or cancel Datadog resources
    #[arg(long)]
    allow_write: bool,

    /// Persist non-sensitive tool responses to the local cache
    #[arg(long)]
    cache_responses: bool,

    /// Delete cached responses older than this many hours at startup
    #[arg(long, default_value_t = 24)]
    cache_retention_hours: u64,

    /// Maximum response-cache size in MiB; oldest files are removed first
    #[arg(long, default_value_t = 100)]
    cache_max_mib: u64,
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
    let _ = dotenvy::dotenv();

    // Initialize tracing to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting Datadog MCP Server");
    info!("Output format: {:?}", args.format);

    // If requested, store credentials in keyring and exit
    #[cfg(feature = "keyring")]
    if args.store_credentials {
        let config = datadog_api::config::DatadogConfig::from_env_or_credentials_file()?;
        config.store_in_keyring()?;
        info!("Stored Datadog credentials in keyring");
        return Ok(());
    }

    // Load Datadog configuration
    let config = datadog_api::config::DatadogConfig::from_env_or_file()?;
    info!("Loaded Datadog configuration for site: {}", config.site);

    let cache_dir = cache::default_cache_dir();
    if args.cache_responses {
        cache::init_cache_in(&cache_dir).await?;
        let deleted = cache::cleanup_cache_in(&cache_dir, args.cache_retention_hours).await?;
        let size_deleted = cache::enforce_cache_size_in(
            &cache_dir,
            args.cache_max_mib.saturating_mul(1024 * 1024),
        )
        .await?;
        info!(
            cache_directory = %cache_dir.display(),
            retention_deleted = deleted,
            size_deleted,
            "Response cache enabled and retention applied"
        );
    }
    info!(
        allow_write = args.allow_write,
        cache_responses = args.cache_responses,
        "Safety mode configured"
    );

    // Initialize server state with output format
    let state = ServerState::new_with_options(
        config,
        args.format,
        ServerOptions {
            allow_write: args.allow_write,
            cache_responses: args.cache_responses,
            cache_dir,
            cache_max_bytes: args.cache_max_mib.saturating_mul(1024 * 1024),
        },
    )
    .await?;
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
