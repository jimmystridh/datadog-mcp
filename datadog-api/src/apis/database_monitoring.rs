//! Documented DBM query metrics, activity samples, and explain-plan samples.
//! https://docs.datadoghq.com/database_monitoring/guide/build_apps_with_dbm_api/

use crate::{DatadogClient, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseSampleType {
    Activity,
    Plan,
}

impl DatabaseSampleType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Plan => "plan",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAggregator {
    Sum,
    Avg,
    Min,
    Max,
    Last,
}

#[derive(Clone)]
pub struct DatabaseMonitoringApi {
    client: DatadogClient,
}

impl DatabaseMonitoringApi {
    pub fn new(client: DatadogClient) -> Self {
        Self { client }
    }

    pub async fn search_samples(
        &self,
        query: &str,
        sample_type: DatabaseSampleType,
        from_timestamp: i64,
        to_timestamp: i64,
        limit: u32,
    ) -> Result<Value> {
        validate_limit(limit, 1000)?;
        let (from, to) = time_range_millis(from_timestamp, to_timestamp)?;
        let query = if query.trim().is_empty() {
            format!("dbm_type:{}", sample_type.as_str())
        } else {
            format!("({query}) AND dbm_type:{}", sample_type.as_str())
        };
        let body = json!({"list": {
            "indexes": ["databasequery"], "limit": limit,
            "search": {"query": query},
            "sorts": [{"time": {"order": "desc"}}],
            "time": {"from": from, "to": to}
        }});
        let data: Value = self
            .client
            .post_app_search("/api/v1/logs-analytics/list?type=databasequery", &body)
            .await
            .map_err(dbm_permission_error)?;
        if !data.pointer("/result/events").is_some_and(Value::is_array) {
            return Err(Error::InvalidResponse(
                "DBM sample response is missing result.events".into(),
            ));
        }
        Ok(data)
    }

    pub async fn query_metrics(
        &self,
        query: &str,
        aggregator: QueryAggregator,
        from_timestamp: i64,
        to_timestamp: i64,
        limit: u32,
    ) -> Result<Value> {
        validate_limit(limit, 1000)?;
        if query.trim().is_empty() {
            return Err(Error::ConfigError(
                "DBM metric query must not be empty".into(),
            ));
        }
        let (from, to) = time_range_millis(from_timestamp, to_timestamp)?;
        let body = json!({"data": {"type": "scalar_request", "attributes": {
            "from": from, "to": to,
            "formulas": [{"formula": "query1", "limit": {"count": limit, "order": "desc"}}],
            "queries": [{"name": "query1", "data_source": "metrics", "query": query, "aggregator": aggregator}]
        }}});
        let data: Value = self
            .client
            .post_search("/api/v2/query/scalar", &body)
            .await?;
        if !data
            .pointer("/data/attributes/columns")
            .is_some_and(Value::is_array)
        {
            return Err(Error::InvalidResponse(
                "DBM metric response is missing data.attributes.columns".into(),
            ));
        }
        Ok(data)
    }
}

fn dbm_permission_error(error: Error) -> Error {
    match error {
        Error::ApiError { status: 403, message } => Error::ApiError {
            status: 403,
            message: format!("{message}. DBM sample APIs require an unscoped application key (built_in_features permission) and Database Monitoring access"),
        },
        error => error,
    }
}

fn validate_limit(limit: u32, max: u32) -> Result<()> {
    if !(1..=max).contains(&limit) {
        return Err(Error::ConfigError(format!(
            "DBM result limit must be between 1 and {max}"
        )));
    }
    Ok(())
}

fn time_range_millis(from: i64, to: i64) -> Result<(i64, i64)> {
    if from < 0 || to <= from {
        return Err(Error::ConfigError(
            "DBM timestamps must be nonnegative Unix seconds with from_timestamp < to_timestamp"
                .into(),
        ));
    }
    let from = from.checked_mul(1000);
    let to = to.checked_mul(1000);
    match (from, to) {
        (Some(from), Some(to)) => Ok((from, to)),
        _ => Err(Error::ConfigError(
            "DBM timestamp overflows milliseconds".into(),
        )),
    }
}
