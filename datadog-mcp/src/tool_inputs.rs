use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// METRICS & MONITORING TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetMetricsInput {
    #[schemars(description = "Metric query")]
    pub query: String,
    #[schemars(description = "Start timestamp (Unix epoch)")]
    pub from_timestamp: i64,
    #[schemars(description = "End timestamp (Unix epoch)")]
    pub to_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchMetricsInput {
    #[schemars(description = "Search pattern for metric names")]
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetMetricMetadataInput {
    #[schemars(description = "Name of the metric")]
    pub metric_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetMonitorInput {
    #[schemars(description = "Monitor ID")]
    pub monitor_id: MonitorId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateMonitorInput {
    #[schemars(description = "Monitor name")]
    pub name: String,
    #[schemars(description = "Monitor type")]
    pub monitor_type: String,
    #[schemars(description = "Monitor query")]
    pub query: String,
    #[schemars(description = "Monitor message")]
    pub message: Option<String>,
    #[schemars(description = "Monitor options")]
    pub options: Option<MonitorOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateMonitorInput {
    #[schemars(description = "Monitor ID")]
    pub monitor_id: MonitorId,
    #[schemars(description = "Monitor name")]
    pub name: Option<String>,
    #[schemars(description = "Monitor query")]
    pub query: Option<String>,
    #[schemars(description = "Monitor message")]
    pub message: Option<String>,
    #[schemars(description = "Monitor options")]
    pub options: Option<MonitorOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteMonitorInput {
    #[schemars(description = "Monitor ID to delete")]
    pub monitor_id: MonitorId,
}

// ============================================================================
// DASHBOARD TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetDashboardInput {
    #[schemars(description = "Dashboard ID")]
    pub dashboard_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDashboardInput {
    #[schemars(description = "Dashboard title")]
    pub title: String,
    #[schemars(description = "Dashboard layout type")]
    pub layout_type: String,
    #[schemars(description = "Dashboard widgets")]
    pub widgets: Vec<serde_json::Value>,
    #[schemars(description = "Dashboard description")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDashboardInput {
    #[schemars(description = "Dashboard ID")]
    pub dashboard_id: String,
    #[schemars(description = "Dashboard title")]
    pub title: Option<String>,
    #[schemars(description = "Dashboard widgets")]
    pub widgets: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteDashboardInput {
    #[schemars(description = "Dashboard ID")]
    pub dashboard_id: String,
}

// ============================================================================
// LOGS & EVENTS TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchLogsInput {
    #[schemars(description = "Log search query")]
    pub query: String,
    #[schemars(description = "Start time")]
    pub from_time: String,
    #[schemars(description = "End time")]
    pub to_time: String,
    #[schemars(description = "Result limit")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetEventsInput {
    #[schemars(description = "Start timestamp")]
    pub start: i64,
    #[schemars(description = "End timestamp")]
    pub end: i64,
    #[schemars(description = "Event priority filter")]
    pub priority: Option<String>,
    #[schemars(description = "Event sources filter")]
    pub sources: Option<String>,
}

// ============================================================================
// INFRASTRUCTURE & TAGS TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTagsInput {
    #[schemars(description = "Tag source filter")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDowntimeInput {
    #[schemars(description = "Downtime scope")]
    pub scope: Vec<String>,
    #[schemars(description = "Start timestamp")]
    pub start: Option<i64>,
    #[schemars(description = "End timestamp")]
    pub end: Option<i64>,
    #[schemars(description = "Downtime message")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelDowntimeInput {
    #[schemars(description = "Downtime ID to cancel")]
    pub downtime_id: i64,
}

// ============================================================================
// INCIDENTS TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetIncidentsInput {
    #[schemars(description = "Page size for pagination")]
    pub page_size: Option<i32>,
}

// ============================================================================
// SYNTHETICS TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateSyntheticsTestInput {
    #[schemars(description = "Name of the Synthetics test")]
    pub name: String,
    #[schemars(description = "Type of test (currently only 'api' is supported for HTTP checks)")]
    pub test_type: String,
    #[schemars(description = "URL to monitor")]
    pub url: String,
    #[schemars(description = "List of location IDs (e.g., ['aws:eu-central-1'])")]
    pub locations: Vec<String>,
    #[schemars(description = "Optional notification message")]
    pub message: Option<String>,
    #[schemars(description = "Optional list of tags")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Test frequency in seconds (default: 300)")]
    pub tick_every: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerSyntheticsTestsInput {
    #[schemars(description = "List of Synthetics test public IDs to trigger")]
    pub test_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSyntheticsTestInput {
    #[schemars(description = "Public ID of the test to update")]
    pub public_id: String,
    #[schemars(description = "Optional new name")]
    pub name: Option<String>,
    #[schemars(description = "Optional new URL")]
    pub url: Option<String>,
    #[schemars(description = "Optional new locations")]
    pub locations: Option<Vec<String>>,
    #[schemars(description = "Optional new message")]
    pub message: Option<String>,
    #[schemars(description = "Optional new tags")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "Test frequency in seconds")]
    pub tick_every: Option<i32>,
}

// ============================================================================
// KUBERNETES TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetKubernetesDeploymentsInput {
    #[schemars(description = "Optional Kubernetes namespace filter")]
    pub namespace: Option<String>,
}

// ============================================================================
// UTILITIES TOOL INPUTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeDataInput {
    #[schemars(description = "File path to analyze")]
    pub filepath: String,
    #[schemars(description = "Analysis type: summary, stats, or trends")]
    pub analysis_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CleanupCacheInput {
    #[schemars(description = "Delete files older than this many hours")]
    pub older_than_hours: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;

    #[test]
    fn test_get_metrics_input_serialization() {
        let input = GetMetricsInput {
            query: "avg:system.cpu.user{*}".to_string(),
            from_timestamp: 1700000000,
            to_timestamp: 1700003600,
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: GetMetricsInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.query, "avg:system.cpu.user{*}");
        assert_eq!(deserialized.from_timestamp, 1700000000);
        assert_eq!(deserialized.to_timestamp, 1700003600);
    }

    #[test]
    fn test_create_monitor_input_with_options() {
        let json = r#"{
            "name": "CPU Alert",
            "monitor_type": "metric alert",
            "query": "avg:system.cpu.user{*} > 80",
            "message": "CPU is high!",
            "options": {"notify_no_data": true}
        }"#;

        let input: CreateMonitorInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, "CPU Alert");
        assert_eq!(input.monitor_type, "metric alert");
        assert!(input.options.is_some());
    }

    #[test]
    fn test_create_monitor_input_minimal() {
        let json = r#"{
            "name": "Test Monitor",
            "monitor_type": "metric alert",
            "query": "avg:test{*}"
        }"#;

        let input: CreateMonitorInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, "Test Monitor");
        assert!(input.message.is_none());
        assert!(input.options.is_none());
    }

    #[test]
    fn test_search_logs_input_with_limit() {
        let input = SearchLogsInput {
            query: "service:api status:error".to_string(),
            from_time: "now-15m".to_string(),
            to_time: "now".to_string(),
            limit: Some(100),
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("\"limit\":100"));
    }

    #[test]
    fn test_create_synthetics_test_input() {
        let input = CreateSyntheticsTestInput {
            name: "API Health Check".to_string(),
            test_type: "api".to_string(),
            url: "https://api.example.com/health".to_string(),
            locations: vec!["aws:eu-central-1".to_string()],
            message: Some("API is down".to_string()),
            tags: Some(vec!["env:prod".to_string()]),
            tick_every: Some(300),
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: CreateSyntheticsTestInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.locations.len(), 1);
        assert_eq!(deserialized.tick_every, Some(300));
    }

    #[test]
    fn test_update_synthetics_test_partial() {
        let json = r#"{
            "public_id": "abc-123-xyz",
            "name": "Updated Name"
        }"#;

        let input: UpdateSyntheticsTestInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.public_id, "abc-123-xyz");
        assert_eq!(input.name, Some("Updated Name".to_string()));
        assert!(input.url.is_none());
        assert!(input.locations.is_none());
    }

    #[test]
    fn test_create_downtime_input() {
        let input = CreateDowntimeInput {
            scope: vec!["env:staging".to_string(), "service:api".to_string()],
            start: Some(1700000000),
            end: Some(1700003600),
            message: Some("Maintenance window".to_string()),
        };

        let json = serde_json::to_string(&input).unwrap();
        assert!(json.contains("\"scope\":["));
        assert!(json.contains("env:staging"));
    }

    #[test]
    fn test_schema_generation() {
        let schema = schema_for!(CreateMonitorInput);
        let json = serde_json::to_string_pretty(&schema).unwrap();

        assert!(json.contains("\"name\""));
        assert!(json.contains("\"monitor_type\""));
        assert!(json.contains("\"query\""));
    }

    #[test]
    fn test_get_kubernetes_deployments_input() {
        let with_ns = GetKubernetesDeploymentsInput {
            namespace: Some("production".to_string()),
        };
        let without_ns = GetKubernetesDeploymentsInput { namespace: None };

        assert!(with_ns.namespace.is_some());
        assert!(without_ns.namespace.is_none());
    }

    #[test]
    fn test_analyze_data_input() {
        let input = AnalyzeDataInput {
            filepath: "/tmp/data.json".to_string(),
            analysis_type: Some("summary".to_string()),
        };

        let json = serde_json::to_string(&input).unwrap();
        let deserialized: AnalyzeDataInput = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.filepath, "/tmp/data.json");
        assert_eq!(deserialized.analysis_type, Some("summary".to_string()));
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct MonitorId(pub i64);

impl std::fmt::Display for MonitorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitorOptions {
    #[schemars(description = "Alerting thresholds")]
    pub thresholds: Option<MonitorThresholds>,
    #[schemars(description = "Notify when no data is received")]
    pub notify_no_data: Option<bool>,
    #[schemars(description = "Evaluation delay in seconds")]
    pub evaluation_delay: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MonitorThresholds {
    pub critical: Option<f64>,
    pub warning: Option<f64>,
    #[serde(rename = "ok")]
    pub ok: Option<f64>,
}

impl From<MonitorThresholds> for datadog_api::models::MonitorThresholds {
    fn from(src: MonitorThresholds) -> Self {
        datadog_api::models::MonitorThresholds {
            critical: src.critical,
            warning: src.warning,
            ok: src.ok,
        }
    }
}

impl From<MonitorOptions> for datadog_api::models::MonitorOptions {
    fn from(src: MonitorOptions) -> Self {
        datadog_api::models::MonitorOptions {
            thresholds: src.thresholds.map(|t| t.into()),
            notify_no_data: src.notify_no_data,
            no_data_timeframe: src.evaluation_delay,
            renotify_interval: None,
            escalation_message: None,
        }
    }
}
