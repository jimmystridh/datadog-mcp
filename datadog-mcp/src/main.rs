mod cache;
mod mcp;
mod tools;
mod tools_part2;

use anyhow::Result;
use datadog_api::{DatadogClient, DatadogConfig};
use mcp::{JsonRpcRequest, JsonRpcResponse, ServerCapabilities, ServerInfo, ToolDefinition};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Datadog MCP Server");

    // Initialize cache
    cache::init_cache().await?;

    // Load Datadog configuration
    let config = DatadogConfig::from_env()?;
    info!("Loaded Datadog configuration for site: {}", config.site);

    // Create Datadog client
    let client = DatadogClient::new(config)?;
    let client = Arc::new(client);

    // Create server
    let mut server = McpServer::new(client);

    info!("Datadog MCP Server is ready");

    // Run the server
    server.run().await?;

    Ok(())
}

struct McpServer {
    client: Arc<DatadogClient>,
    tools: HashMap<String, ToolDefinition>,
}

impl McpServer {
    fn new(client: Arc<DatadogClient>) -> Self {
        let tools = Self::register_tools();
        Self { client, tools }
    }

    fn register_tools() -> HashMap<String, ToolDefinition> {
        let mut tools = HashMap::new();

        // Metrics & Monitoring (9 tools)
        tools.insert("validate_api_key".to_string(), ToolDefinition {
            name: "validate_api_key".to_string(),
            description: "Validate Datadog API credentials".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_metrics".to_string(), ToolDefinition {
            name: "get_metrics".to_string(),
            description: "Query Datadog metrics time series data".to_string(),
            input_schema: json!({"type": "object", "properties": {"query": {"type": "string", "description": "Metric query"}, "from_timestamp": {"type": "integer", "description": "Start timestamp (Unix epoch)"}, "to_timestamp": {"type": "integer", "description": "End timestamp (Unix epoch)"}}, "required": ["query", "from_timestamp", "to_timestamp"]}),
        });
        tools.insert("search_metrics".to_string(), ToolDefinition {
            name: "search_metrics".to_string(),
            description: "Search for metrics by name pattern".to_string(),
            input_schema: json!({"type": "object", "properties": {"query": {"type": "string", "description": "Search pattern for metric names"}}, "required": ["query"]}),
        });
        tools.insert("get_metric_metadata".to_string(), ToolDefinition {
            name: "get_metric_metadata".to_string(),
            description: "Get metadata for a specific metric".to_string(),
            input_schema: json!({"type": "object", "properties": {"metric_name": {"type": "string", "description": "Name of the metric"}}, "required": ["metric_name"]}),
        });
        tools.insert("get_monitors".to_string(), ToolDefinition {
            name: "get_monitors".to_string(),
            description: "Get all Datadog monitors".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_monitor".to_string(), ToolDefinition {
            name: "get_monitor".to_string(),
            description: "Get specific monitor by ID".to_string(),
            input_schema: json!({"type": "object", "properties": {"monitor_id": {"type": "integer", "description": "Monitor ID"}}, "required": ["monitor_id"]}),
        });
        tools.insert("create_monitor".to_string(), ToolDefinition {
            name: "create_monitor".to_string(),
            description: "Create a new Datadog monitor".to_string(),
            input_schema: json!({"type": "object", "properties": {"name": {"type": "string", "description": "Monitor name"}, "type": {"type": "string", "description": "Monitor type"}, "query": {"type": "string", "description": "Monitor query"}, "message": {"type": "string"}, "options": {"type": "object"}}, "required": ["name", "type", "query"]}),
        });
        tools.insert("update_monitor".to_string(), ToolDefinition {
            name: "update_monitor".to_string(),
            description: "Update an existing Datadog monitor".to_string(),
            input_schema: json!({"type": "object", "properties": {"monitor_id": {"type": "integer", "description": "Monitor ID"}, "name": {"type": "string"}, "query": {"type": "string"}, "message": {"type": "string"}, "options": {"type": "object"}}, "required": ["monitor_id"]}),
        });
        tools.insert("delete_monitor".to_string(), ToolDefinition {
            name: "delete_monitor".to_string(),
            description: "Delete a monitor".to_string(),
            input_schema: json!({"type": "object", "properties": {"monitor_id": {"type": "integer", "description": "Monitor ID to delete"}}, "required": ["monitor_id"]}),
        });

        // Dashboards (5 tools)
        tools.insert("get_dashboards".to_string(), ToolDefinition {
            name: "get_dashboards".to_string(),
            description: "Get all Datadog dashboards".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_dashboard".to_string(), ToolDefinition {
            name: "get_dashboard".to_string(),
            description: "Get specific dashboard by ID".to_string(),
            input_schema: json!({"type": "object", "properties": {"dashboard_id": {"type": "string", "description": "Dashboard ID"}}, "required": ["dashboard_id"]}),
        });
        tools.insert("create_dashboard".to_string(), ToolDefinition {
            name: "create_dashboard".to_string(),
            description: "Create a new dashboard".to_string(),
            input_schema: json!({"type": "object", "properties": {"title": {"type": "string"}, "layout_type": {"type": "string"}, "widgets": {"type": "array"}, "description": {"type": "string"}}, "required": ["title", "layout_type", "widgets"]}),
        });
        tools.insert("update_dashboard".to_string(), ToolDefinition {
            name: "update_dashboard".to_string(),
            description: "Update an existing dashboard".to_string(),
            input_schema: json!({"type": "object", "properties": {"dashboard_id": {"type": "string"}, "title": {"type": "string"}, "widgets": {"type": "array"}}, "required": ["dashboard_id"]}),
        });
        tools.insert("delete_dashboard".to_string(), ToolDefinition {
            name: "delete_dashboard".to_string(),
            description: "Delete a dashboard".to_string(),
            input_schema: json!({"type": "object", "properties": {"dashboard_id": {"type": "string"}}, "required": ["dashboard_id"]}),
        });

        // Logs & Events (2 tools)
        tools.insert("search_logs".to_string(), ToolDefinition {
            name: "search_logs".to_string(),
            description: "Search Datadog logs".to_string(),
            input_schema: json!({"type": "object", "properties": {"query": {"type": "string"}, "from_time": {"type": "string"}, "to_time": {"type": "string"}, "limit": {"type": "integer"}}, "required": ["query", "from_time", "to_time"]}),
        });
        tools.insert("get_events".to_string(), ToolDefinition {
            name: "get_events".to_string(),
            description: "Get Datadog events".to_string(),
            input_schema: json!({"type": "object", "properties": {"start": {"type": "integer"}, "end": {"type": "integer"}, "priority": {"type": "string"}, "sources": {"type": "string"}}, "required": ["start", "end"]}),
        });

        // Infrastructure & Tags (4 tools - note: get_service_map was disabled in Python)
        tools.insert("get_infrastructure".to_string(), ToolDefinition {
            name: "get_infrastructure".to_string(),
            description: "Get infrastructure and hosts information".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_tags".to_string(), ToolDefinition {
            name: "get_tags".to_string(),
            description: "Get host tags".to_string(),
            input_schema: json!({"type": "object", "properties": {"source": {"type": "string"}}, "required": []}),
        });
        tools.insert("get_downtimes".to_string(), ToolDefinition {
            name: "get_downtimes".to_string(),
            description: "Get scheduled downtimes".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("create_downtime".to_string(), ToolDefinition {
            name: "create_downtime".to_string(),
            description: "Create a scheduled downtime".to_string(),
            input_schema: json!({"type": "object", "properties": {"scope": {"type": "array", "items": {"type": "string"}}, "start": {"type": "integer"}, "end": {"type": "integer"}, "message": {"type": "string"}}, "required": ["scope"]}),
        });

        // Testing (1 tool - note: get_rum_applications was disabled in Python)
        tools.insert("get_synthetics_tests".to_string(), ToolDefinition {
            name: "get_synthetics_tests".to_string(),
            description: "Get all Synthetics tests".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });

        // Security & Incidents (4 tools)
        tools.insert("get_security_rules".to_string(), ToolDefinition {
            name: "get_security_rules".to_string(),
            description: "Get security monitoring rules".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_incidents".to_string(), ToolDefinition {
            name: "get_incidents".to_string(),
            description: "Get incidents with pagination support".to_string(),
            input_schema: json!({"type": "object", "properties": {"page_size": {"type": "integer"}}, "required": []}),
        });
        tools.insert("get_slos".to_string(), ToolDefinition {
            name: "get_slos".to_string(),
            description: "Get Service Level Objectives".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_notebooks".to_string(), ToolDefinition {
            name: "get_notebooks".to_string(),
            description: "Get Datadog notebooks".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });

        // Teams & Users (2 tools)
        tools.insert("get_teams".to_string(), ToolDefinition {
            name: "get_teams".to_string(),
            description: "Get teams".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });
        tools.insert("get_users".to_string(), ToolDefinition {
            name: "get_users".to_string(),
            description: "Get users".to_string(),
            input_schema: json!({"type": "object", "properties": {}, "required": []}),
        });

        // Utilities (2 tools)
        tools.insert("analyze_data".to_string(), ToolDefinition {
            name: "analyze_data".to_string(),
            description: "Analyze stored Datadog data (summary, stats, or trends)".to_string(),
            input_schema: json!({"type": "object", "properties": {"filepath": {"type": "string"}, "analysis_type": {"type": "string", "enum": ["summary", "stats", "trends"]}}, "required": ["filepath"]}),
        });
        tools.insert("cleanup_cache".to_string(), ToolDefinition {
            name: "cleanup_cache".to_string(),
            description: "Clean up old cache files".to_string(),
            input_schema: json!({"type": "object", "properties": {"older_than_hours": {"type": "integer"}}, "required": []}),
        });

        tools
    }

    async fn run(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;

            if n == 0 {
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            debug!("Received: {}", trimmed);

            let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                Ok(request) => self.handle_request(request).await,
                Err(e) => JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e)),
            };

            let response_str = serde_json::to_string(&response)?;
            stdout.write_all(response_str.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tool_call(request.id, request.params).await,
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    fn handle_initialize(&self, id: Option<Value>) -> JsonRpcResponse {
        let result = json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": ServerInfo {
                name: "datadog-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            "capabilities": ServerCapabilities {
                tools: Some(HashMap::new()),
            },
        });

        JsonRpcResponse::success(id, result)
    }

    fn handle_tools_list(&self, id: Option<Value>) -> JsonRpcResponse {
        let tools: Vec<_> = self.tools.values().cloned().collect();
        let result = json!({ "tools": tools });
        JsonRpcResponse::success(id, result)
    }

    async fn handle_tool_call(&self, id: Option<Value>, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => return JsonRpcResponse::error(id, -32602, "Invalid params".to_string()),
        };

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name,
            None => return JsonRpcResponse::error(id, -32602, "Missing tool name".to_string()),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = self.execute_tool(tool_name, arguments).await;

        match result {
            Ok(output) => JsonRpcResponse::success(id, json!({ "content": [{"type": "text", "text": serde_json::to_string_pretty(&output).unwrap_or_default()}] })),
            Err(e) => JsonRpcResponse::error(id, -32603, format!("Tool execution failed: {}", e)),
        }
    }

    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value> {
        match name {
            "validate_api_key" => tools::validate_api_key(self.client.clone()).await,
            "get_metrics" => {
                let query = args["query"].as_str().unwrap_or("").to_string();
                let from = args["from_timestamp"].as_i64().unwrap_or(0);
                let to = args["to_timestamp"].as_i64().unwrap_or(0);
                tools::get_metrics(self.client.clone(), query, from, to).await
            }
            "search_metrics" => {
                let query = args["query"].as_str().unwrap_or("").to_string();
                tools::search_metrics(self.client.clone(), query).await
            }
            "get_metric_metadata" => {
                let metric_name = args["metric_name"].as_str().unwrap_or("").to_string();
                tools::get_metric_metadata(self.client.clone(), metric_name).await
            }
            "get_monitors" => tools::get_monitors(self.client.clone()).await,
            "get_monitor" => {
                let monitor_id = args["monitor_id"].as_i64().unwrap_or(0);
                tools::get_monitor(self.client.clone(), monitor_id).await
            }
            "create_monitor" => {
                let name = args["name"].as_str().unwrap_or("").to_string();
                let monitor_type = args["type"].as_str().unwrap_or("").to_string();
                let query = args["query"].as_str().unwrap_or("").to_string();
                let message = args.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                let options = args.get("options").cloned();
                tools::create_monitor(self.client.clone(), name, monitor_type, query, message, options).await
            }
            "update_monitor" => {
                let monitor_id = args["monitor_id"].as_i64().unwrap_or(0);
                let name = args.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                let query = args.get("query").and_then(|v| v.as_str()).map(|s| s.to_string());
                let message = args.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                let options = args.get("options").cloned();
                tools::update_monitor(self.client.clone(), monitor_id, name, query, message, options).await
            }
            "delete_monitor" => {
                let monitor_id = args["monitor_id"].as_i64().unwrap_or(0);
                tools::delete_monitor(self.client.clone(), monitor_id).await
            }
            "get_dashboards" => tools::get_dashboards(self.client.clone()).await,
            "get_dashboard" => {
                let dashboard_id = args["dashboard_id"].as_str().unwrap_or("").to_string();
                tools::get_dashboard(self.client.clone(), dashboard_id).await
            }
            "create_dashboard" => {
                let title = args["title"].as_str().unwrap_or("").to_string();
                let layout_type = args["layout_type"].as_str().unwrap_or("ordered").to_string();
                let widgets = args["widgets"].as_array().cloned().unwrap_or_default();
                let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                tools::create_dashboard(self.client.clone(), title, layout_type, widgets, description).await
            }
            "update_dashboard" => {
                let dashboard_id = args["dashboard_id"].as_str().unwrap_or("").to_string();
                let title = args.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());
                let widgets = args.get("widgets").and_then(|v| v.as_array()).cloned();
                tools::update_dashboard(self.client.clone(), dashboard_id, title, widgets).await
            }
            "delete_dashboard" => {
                let dashboard_id = args["dashboard_id"].as_str().unwrap_or("").to_string();
                tools::delete_dashboard(self.client.clone(), dashboard_id).await
            }
            "search_logs" => {
                let query = args["query"].as_str().unwrap_or("").to_string();
                let from_time = args["from_time"].as_str().unwrap_or("").to_string();
                let to_time = args["to_time"].as_str().unwrap_or("").to_string();
                let limit = args.get("limit").and_then(|v| v.as_i64()).map(|v| v as i32);
                tools_part2::search_logs(self.client.clone(), query, from_time, to_time, limit).await
            }
            "get_events" => {
                let start = args["start"].as_i64().unwrap_or(0);
                let end = args["end"].as_i64().unwrap_or(0);
                let priority = args.get("priority").and_then(|v| v.as_str()).map(|s| s.to_string());
                let sources = args.get("sources").and_then(|v| v.as_str()).map(|s| s.to_string());
                tools_part2::get_events(self.client.clone(), start, end, priority, sources).await
            }
            "get_infrastructure" => tools_part2::get_infrastructure(self.client.clone()).await,
            "get_tags" => {
                let source = args.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
                tools_part2::get_tags(self.client.clone(), source).await
            }
            "get_downtimes" => tools_part2::get_downtimes(self.client.clone()).await,
            "create_downtime" => {
                let scope = args["scope"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let start = args.get("start").and_then(|v| v.as_i64());
                let end = args.get("end").and_then(|v| v.as_i64());
                let message = args.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                tools_part2::create_downtime(self.client.clone(), scope, start, end, message).await
            }
            "get_synthetics_tests" => tools_part2::get_synthetics_tests(self.client.clone()).await,
            "get_security_rules" => tools_part2::get_security_rules(self.client.clone()).await,
            "get_incidents" => {
                let page_size = args.get("page_size").and_then(|v| v.as_i64()).map(|v| v as i32);
                tools_part2::get_incidents(self.client.clone(), page_size).await
            }
            "get_slos" => tools_part2::get_slos(self.client.clone()).await,
            "get_notebooks" => tools_part2::get_notebooks(self.client.clone()).await,
            "get_teams" => tools_part2::get_teams(self.client.clone()).await,
            "get_users" => tools_part2::get_users(self.client.clone()).await,
            "analyze_data" => {
                let filepath = args["filepath"].as_str().unwrap_or("").to_string();
                let analysis_type = args.get("analysis_type").and_then(|v| v.as_str()).map(|s| s.to_string());
                tools_part2::analyze_data(filepath, analysis_type).await
            }
            "cleanup_cache" => {
                let older_than_hours = args.get("older_than_hours").and_then(|v| v.as_u64());
                tools_part2::cleanup_cache_tool(older_than_hours).await
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }
}
