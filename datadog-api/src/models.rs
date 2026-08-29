use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Common types
pub type Tags = Vec<String>;
pub type JsonValue = serde_json::Value;

// ============================================================================
// DASHBOARD WIDGETS
// ============================================================================

/// Common widget definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    /// Widget definition containing type-specific configuration
    pub definition: WidgetDefinition,
    /// Widget layout (for free-form dashboards)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<WidgetLayout>,
    /// Widget ID (set by Datadog)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

/// Widget layout for free-form dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetLayout {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

/// Widget definition - uses tagged enum for type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetDefinition {
    /// Timeseries graph widget
    Timeseries(TimeseriesDefinition),
    /// Single value query widget
    QueryValue(QueryValueDefinition),
    /// Top list widget
    Toplist(ToplistDefinition),
    /// Table widget
    QueryTable(QueryTableDefinition),
    /// Heatmap widget
    Heatmap(HeatmapDefinition),
    /// Note/text widget
    Note(NoteDefinition),
    /// Group widget (contains other widgets)
    Group(GroupDefinition),
    /// Fallback for unsupported widget types
    #[serde(other, rename = "unknown")]
    Unknown,
}

/// Timeseries widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesDefinition {
    pub requests: Vec<TimeseriesRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_legend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend_size: Option<String>,
}

/// Request for timeseries data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formulas: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<JsonValue>,
}

/// Query value widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryValueDefinition {
    pub requests: Vec<QueryValueRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoscale: Option<bool>,
}

/// Request for query value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryValueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formulas: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregator: Option<String>,
}

/// Toplist widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToplistDefinition {
    pub requests: Vec<ToplistRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Request for toplist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToplistRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formulas: Option<Vec<JsonValue>>,
}

/// Query table widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTableDefinition {
    pub requests: Vec<QueryTableRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Request for query table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTableRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<Vec<JsonValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formulas: Option<Vec<JsonValue>>,
}

/// Heatmap widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapDefinition {
    pub requests: Vec<HeatmapRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Request for heatmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<JsonValue>,
}

/// Note/text widget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDefinition {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_tick: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_pos: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_edge: Option<String>,
}

/// Group widget definition (contains nested widgets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDefinition {
    pub layout_type: String,
    pub widgets: Vec<Widget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsQueryResponse {
    pub series: Option<Vec<MetricSeries>>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesFormulaQueryResponse {
    pub data: Option<TimeseriesFormulaResponseData>,
    pub errors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesFormulaResponseData {
    pub attributes: TimeseriesFormulaResponseAttributes,
    #[serde(rename = "type")]
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesFormulaResponseAttributes {
    #[serde(default)]
    pub series: Vec<TimeseriesFormulaResponseSeries>,
    #[serde(default)]
    pub times: Vec<i64>,
    #[serde(default)]
    pub values: Vec<Vec<Option<f64>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesFormulaResponseSeries {
    #[serde(default)]
    pub group_tags: Vec<String>,
    pub query_index: Option<i32>,
    pub unit: Option<Vec<Option<MetricUnit>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub metric: Option<String>,
    pub pointlist: Option<Vec<Vec<serde_json::Value>>>,
    pub scope: Option<String>,
    pub display_name: Option<String>,
    pub unit: Option<Vec<Option<MetricUnit>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricUnit {
    pub family: Option<String>,
    pub name: Option<String>,
    pub plural: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub scale_factor: Option<f64>,
    pub short_name: Option<String>,
}

fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("scale factor is not representable as f64"))
            .map(Some),
        Some(serde_json::Value::String(value)) if value.is_empty() => Ok(None),
        Some(value) => Err(serde::de::Error::custom(format!(
            "invalid scale factor: {value}"
        ))),
    }
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
    pub evaluation_delay: Option<i64>,
    pub renotify_interval: Option<i64>,
    pub escalation_message: Option<String>,
    pub include_tags: Option<bool>,
    pub require_full_window: Option<bool>,
    pub new_group_delay: Option<i64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchResponse {
    pub counts: Option<MonitorSearchCounts>,
    pub metadata: Option<MonitorSearchMetadata>,
    pub monitors: Option<Vec<MonitorSearchResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchCounts {
    pub muted: Option<Vec<MonitorSearchFacet>>,
    pub status: Option<Vec<MonitorSearchFacet>>,
    #[serde(rename = "tag")]
    pub tags: Option<Vec<MonitorSearchFacet>>,
    #[serde(rename = "type")]
    pub types: Option<Vec<MonitorSearchFacet>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchFacet {
    pub count: Option<i64>,
    pub name: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchMetadata {
    pub page: Option<i64>,
    pub page_count: Option<i64>,
    pub per_page: Option<i64>,
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchResult {
    pub classification: Option<String>,
    pub creator: Option<Creator>,
    pub id: Option<i64>,
    pub last_triggered_ts: Option<i64>,
    pub metrics: Option<Vec<String>>,
    pub name: Option<String>,
    pub notifications: Option<Vec<MonitorSearchNotification>>,
    pub org_id: Option<i64>,
    pub quality_issues: Option<Vec<String>>,
    pub query: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub monitor_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorSearchNotification {
    pub handle: Option<String>,
    pub name: Option<String>,
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
    pub widgets: Option<Vec<Widget>>,
    pub layout_type: Option<String>,
    pub is_read_only: Option<bool>,
    pub notify_list: Option<Vec<String>>,
    pub template_variables: Option<Vec<TemplateVariable>>,
}

/// Dashboard template variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_values: Option<Vec<String>>,
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
    pub id_str: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub date_happened: Option<i64>,
    pub device_name: Option<String>,
    pub host: Option<String>,
    pub tags: Option<Tags>,
    pub priority: Option<String>,
    pub alert_type: Option<String>,
    pub source_type_name: Option<String>,
    pub payload: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCreateRequest {
    pub title: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_happened: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_event_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub event: Option<Event>,
    pub status: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsLocation {
    pub id: String,
    pub name: String,
    pub is_private: Option<bool>,
    pub region: Option<SyntheticsRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsRegion {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsLocationsResponse {
    pub locations: Vec<SyntheticsLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestRequest {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyntheticsAssertionType {
    StatusCode,
    ResponseTime,
    Body,
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SyntheticsAssertionOperator {
    Is,
    IsNot,
    LessThan,
    GreaterThan,
    Contains,
    DoesNotContain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsAssertion {
    #[serde(rename = "type")]
    pub assertion_type: SyntheticsAssertionType,
    pub operator: SyntheticsAssertionOperator,
    pub target: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestConfig {
    pub request: SyntheticsTestRequest,
    pub assertions: Vec<SyntheticsAssertion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestOptions {
    pub tick_every: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_failure_duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_location_failed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<SyntheticsRetry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsRetry {
    pub count: i32,
    pub interval: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyntheticsTestType {
    Api,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyntheticsTestSubtype {
    Http,
    Ssl,
    Tcp,
    Dns,
    Multi,
    Grpc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestCreateRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub test_type: SyntheticsTestType,
    pub subtype: SyntheticsTestSubtype,
    pub config: SyntheticsTestConfig,
    pub options: SyntheticsTestOptions,
    pub locations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTestResponse {
    pub public_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub test_type: SyntheticsTestType,
    pub subtype: SyntheticsTestSubtype,
    pub config: SyntheticsTestConfig,
    pub options: SyntheticsTestOptions,
    pub locations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTriggerRequest {
    pub tests: Vec<SyntheticsTriggerTest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTriggerTest {
    pub public_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTriggerResponse {
    pub triggered_check_ids: Vec<String>,
    pub results: Vec<SyntheticsTriggerResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsTriggerResult {
    pub public_id: String,
    pub result_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsDeleteTestsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_delete_dependencies: Option<bool>,
    pub public_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsDeleteTestsResponse {
    pub deleted_tests: Option<Vec<SyntheticsDeletedTest>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticsDeletedTest {
    pub deleted_at: Option<String>,
    pub public_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersV2Response {
    pub data: Option<Vec<UserV2>>,
    pub meta: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserV2 {
    pub id: Option<String>,
    pub attributes: Option<JsonValue>,
}

// APM/Spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchRequest {
    pub data: SpansSearchData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchData {
    pub attributes: SpansSearchRequestAttributes,
    #[serde(rename = "type")]
    pub resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchRequestAttributes {
    pub filter: SpansSearchFilter,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<SpansSearchPage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchFilter {
    pub from: String,
    pub query: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpansSearchResponse {
    pub data: Option<Vec<JsonValue>>,
    pub links: Option<JsonValue>,
    pub meta: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApmServiceDependency {
    pub calls: Vec<String>,
}

pub type ApmServiceDependenciesResponse = HashMap<String, ApmServiceDependency>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceListResponse {
    pub data: JsonValue,
}
