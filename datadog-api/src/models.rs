use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Common types
pub type Tags = Vec<String>;
pub type JsonValue = serde_json::Value;

// Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQueryResponse {
    pub series: Option<Vec<MetricSeries>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub metric: Option<String>,
    pub pointlist: Option<Vec<Vec<serde_json::Value>>>,
    pub scope: Option<String>,
    pub display_name: Option<String>,
    pub unit: Option<Vec<MetricUnit>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricUnit {
    pub family: Option<String>,
    pub name: Option<String>,
    pub plural: Option<String>,
    pub scale_factor: Option<f64>,
    pub short_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsListResponse {
    pub metrics: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricMetadata {
    pub description: Option<String>,
    pub short_name: Option<String>,
    #[serde(rename = "type")]
    pub metric_type: Option<String>,
    pub unit: Option<String>,
    pub per_unit: Option<String>,
    pub statsd_interval: Option<i64>,
}

// Monitors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub monitor_type: Option<String>,
    pub query: Option<String>,
    pub message: Option<String>,
    pub tags: Option<Tags>,
    pub options: Option<MonitorOptions>,
    pub overall_state: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub creator: Option<Creator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorOptions {
    pub thresholds: Option<MonitorThresholds>,
    pub notify_no_data: Option<bool>,
    pub no_data_timeframe: Option<i64>,
    pub renotify_interval: Option<i64>,
    pub escalation_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorThresholds {
    pub critical: Option<f64>,
    pub warning: Option<f64>,
    pub ok: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creator {
    pub email: Option<String>,
    pub handle: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorCreateRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub monitor_type: String,
    pub query: String,
    pub message: Option<String>,
    pub tags: Option<Tags>,
    pub options: Option<MonitorOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorUpdateRequest {
    pub name: Option<String>,
    pub query: Option<String>,
    pub message: Option<String>,
    pub tags: Option<Tags>,
    pub options: Option<MonitorOptions>,
}

// Dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardListResponse {
    pub dashboards: Option<Vec<DashboardSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub author_handle: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub url: Option<String>,
    pub layout_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub widgets: Option<Vec<JsonValue>>,
    pub layout_type: Option<String>,
    pub is_read_only: Option<bool>,
    pub notify_list: Option<Vec<String>>,
    pub template_variables: Option<Vec<JsonValue>>,
}

// Logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsSearchRequest {
    pub filter: LogsFilter,
    pub page: Option<LogsPage>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsFilter {
    pub query: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsPage {
    pub limit: Option<i32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsSearchResponse {
    pub data: Option<Vec<LogEntry>>,
    pub meta: Option<LogsMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<String>,
    pub attributes: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsMeta {
    pub page: Option<LogsPageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsPageInfo {
    pub after: Option<String>,
}

// Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsResponse {
    pub events: Option<Vec<Event>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub date_happened: Option<i64>,
    pub tags: Option<Tags>,
    pub priority: Option<String>,
    pub alert_type: Option<String>,
}

// Infrastructure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsResponse {
    pub host_list: Option<Vec<Host>>,
    pub total_matching: Option<i64>,
    pub total_returned: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub up: Option<bool>,
    pub last_reported_time: Option<i64>,
    pub tags_by_source: Option<HashMap<String, Tags>>,
    pub meta: Option<HostMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMeta {
    pub agent_version: Option<String>,
    pub cpu_cores: Option<i64>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagsResponse {
    pub tags: Option<HashMap<String, Tags>>,
}

// Downtimes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downtime {
    pub id: Option<i64>,
    pub scope: Option<Tags>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub message: Option<String>,
    pub active: Option<bool>,
    pub canceled: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DowntimeCreateRequest {
    pub scope: Tags,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub message: Option<String>,
}

// Synthetics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestsResponse {
    pub tests: Option<Vec<SyntheticsTest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTest {
    pub public_id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub test_type: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Tags>,
}

// Security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRulesResponse {
    pub data: Option<Vec<SecurityRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub id: Option<String>,
    pub attributes: Option<SecurityRuleAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRuleAttributes {
    pub name: Option<String>,
    #[serde(rename = "isEnabled")]
    pub is_enabled: Option<bool>,
    pub message: Option<String>,
}

// Incidents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentsResponse {
    pub data: Option<Vec<Incident>>,
    pub meta: Option<IncidentsMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: Option<String>,
    pub attributes: Option<IncidentAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentAttributes {
    pub title: Option<String>,
    pub state: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentsMeta {
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub next_offset: Option<i64>,
    pub size: Option<i64>,
}

// SLOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOsResponse {
    pub data: Option<Vec<SLO>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLO {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Tags>,
    pub thresholds: Option<Vec<SLOThreshold>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLOThreshold {
    pub target: Option<f64>,
    pub timeframe: Option<String>,
    pub warning: Option<f64>,
}

// Notebooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebooksResponse {
    pub data: Option<Vec<Notebook>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: Option<i64>,
    pub attributes: Option<NotebookAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookAttributes {
    pub name: Option<String>,
    pub author: Option<Creator>,
    pub cells: Option<Vec<JsonValue>>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

// Teams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsResponse {
    pub data: Option<Vec<Team>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Option<String>,
    pub attributes: Option<TeamAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAttributes {
    pub name: Option<String>,
    pub handle: Option<String>,
}

// Users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    pub users: Option<Vec<User>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub handle: Option<String>,
    pub verified: Option<bool>,
}
