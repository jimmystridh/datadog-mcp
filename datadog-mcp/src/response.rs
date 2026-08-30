use crate::state::ToolContext;
use rmcp::model::{CallToolResult, ContentBlock};
use serde::Serialize;
use serde_json::{json, Map, Value};
use tracing::{error, info, warn};

pub const RESPONSE_SIZE_WARN_THRESHOLD: usize = 1024 * 1024;
pub const RESPONSE_SIZE_MAX: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    Never,
    Store(&'static str),
}

#[derive(Debug)]
pub struct ToolOutput {
    data: Option<Value>,
    summary: String,
    fields: Map<String, Value>,
    cache_policy: CachePolicy,
}

impl ToolOutput {
    const RESERVED_FIELDS: [&'static str; 4] = ["status", "summary", "filepath", "data"];

    pub fn from_data<T: Serialize>(
        data: &T,
        summary: impl Into<String>,
        fields: Value,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<Self> {
        let fields = fields
            .as_object()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("tool response fields must be a JSON object"))?;
        Self::validate_fields(&fields)?;
        Ok(Self {
            data: Some(serde_json::to_value(data)?),
            summary: summary.into(),
            fields,
            cache_policy,
        })
    }

    pub fn without_data(summary: impl Into<String>, fields: Value) -> anyhow::Result<Self> {
        let fields = fields
            .as_object()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("tool response fields must be a JSON object"))?;
        Self::validate_fields(&fields)?;
        Ok(Self {
            data: None,
            summary: summary.into(),
            fields,
            cache_policy: CachePolicy::Never,
        })
    }

    fn validate_fields(fields: &Map<String, Value>) -> anyhow::Result<()> {
        if let Some(field) = Self::RESERVED_FIELDS
            .into_iter()
            .find(|field| fields.contains_key(*field))
        {
            anyhow::bail!("tool response field '{field}' is reserved");
        }
        Ok(())
    }

    async fn into_call_result(self, context: &ToolContext) -> anyhow::Result<CallToolResult> {
        if let Some(data) = &self.data {
            let size = serde_json::to_vec(data)?.len();
            if size > RESPONSE_SIZE_MAX {
                anyhow::bail!(
                    "response is {size} bytes, exceeding the {RESPONSE_SIZE_MAX}-byte limit; use pagination or a narrower query"
                );
            }
            if size > RESPONSE_SIZE_WARN_THRESHOLD {
                warn!(size, "Large tool response; consider pagination");
            }
        }

        let filepath = match (self.cache_policy, &self.data) {
            (CachePolicy::Store(prefix), Some(data)) => {
                context
                    .cache
                    .store(data, prefix, context.output_format)
                    .await?
            }
            (CachePolicy::Never, _) | (CachePolicy::Store(_), None) => None,
        };

        info!(summary = %self.summary, "Datadog MCP tool succeeded");
        let mut structured = Map::from_iter([
            ("status".to_string(), Value::String("success".to_string())),
            ("summary".to_string(), Value::String(self.summary)),
            ("filepath".to_string(), json!(filepath)),
        ]);
        if let Some(data) = self.data {
            structured.insert("data".to_string(), data);
        }
        structured.extend(self.fields);
        let structured = Value::Object(structured);
        let text = context.output_format.format(&structured)?;
        let mut result = CallToolResult::structured(structured);
        result.content = vec![ContentBlock::text(text)];
        Ok(result)
    }
}

pub async fn render_tool_result(
    result: anyhow::Result<ToolOutput>,
    context: &ToolContext,
) -> CallToolResult {
    match result {
        Ok(output) => match output.into_call_result(context).await {
            Ok(result) => result,
            Err(error) => error_result(error, context),
        },
        Err(error) => error_result(error, context),
    }
}

fn error_result(error: anyhow::Error, context: &ToolContext) -> CallToolResult {
    error!(tool_error = %error, "Datadog MCP tool failed");
    let structured = json!({
        "status": "error",
        "error": error.to_string(),
    });
    let text = context
        .output_format
        .format(&structured)
        .unwrap_or_else(|_| structured.to_string());
    let mut result = CallToolResult::structured_error(structured);
    result.content = vec![ContentBlock::text(text)];
    result
}

pub fn simple_success(summary: impl Into<String>) -> anyhow::Result<ToolOutput> {
    ToolOutput::without_data(summary, json!({}))
}

pub fn simple_success_with_fields(
    summary: impl Into<String>,
    fields: Value,
) -> anyhow::Result<ToolOutput> {
    ToolOutput::without_data(summary, fields)
}

#[macro_export]
macro_rules! tool_response_with_fields {
    ($result:expr, cache($prefix:literal), $data_ident:ident, $summary:expr, $fields:block) => {
        match $result {
            Ok($data_ident) => $crate::response::ToolOutput::from_data(
                &$data_ident,
                $summary,
                $fields,
                $crate::response::CachePolicy::Store($prefix),
            ),
            Err(error) => Err(error.into()),
        }
    };
    ($result:expr, no_cache, $data_ident:ident, $summary:expr, $fields:block) => {
        match $result {
            Ok($data_ident) => $crate::response::ToolOutput::from_data(
                &$data_ident,
                $summary,
                $fields,
                $crate::response::CachePolicy::Never,
            ),
            Err(error) => Err(error.into()),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheStore;
    use crate::output::OutputFormat;
    use datadog_api::{DatadogClient, DatadogConfig};
    use std::sync::Arc;

    #[test]
    fn typed_output_rejects_non_object_fields() {
        let result = ToolOutput::from_data(
            &json!({"value": 1}),
            "summary",
            json!([]),
            CachePolicy::Never,
        );
        assert!(result.is_err());
    }

    #[test]
    fn typed_output_rejects_reserved_fields() {
        let result = ToolOutput::without_data("summary", json!({ "status": "created" }));
        assert!(result.is_err());
    }

    #[test]
    fn response_size_thresholds_are_ordered() {
        const { assert!(RESPONSE_SIZE_WARN_THRESHOLD < RESPONSE_SIZE_MAX) };
    }

    #[tokio::test]
    async fn renderer_uses_the_configured_cache_store() {
        let directory = tempfile::tempdir().unwrap();
        let cache = CacheStore::new(directory.path().to_path_buf(), u64::MAX, true);
        let client = DatadogClient::new(DatadogConfig::new("api".into(), "app".into())).unwrap();
        let context = ToolContext::with_cache(Arc::new(client), OutputFormat::Json, cache);
        let output = ToolOutput::from_data(
            &json!({ "value": 1 }),
            "cached",
            json!({}),
            CachePolicy::Store("configured"),
        )
        .unwrap();

        let rendered = render_tool_result(Ok(output), &context).await;
        let filepath = rendered.structured_content.unwrap()["filepath"]
            .as_str()
            .unwrap()
            .to_string();

        assert!(filepath.starts_with(directory.path().to_str().unwrap()));
        assert!(std::path::Path::new(&filepath).is_file());
    }
}
