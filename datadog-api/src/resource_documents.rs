use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Lossless dashboard representation used for read-modify-write operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DashboardDocument(Map<String, Value>);

#[derive(Debug, Default)]
pub struct DashboardPatch {
    pub title: Option<String>,
    pub widgets: Option<Vec<Value>>,
}

impl DashboardDocument {
    pub fn new(
        title: String,
        layout_type: String,
        widgets: Vec<Value>,
        description: Option<String>,
    ) -> Self {
        Self(Map::from_iter([
            ("title".to_string(), json!(title)),
            ("layout_type".to_string(), json!(layout_type)),
            ("widgets".to_string(), json!(widgets)),
            ("description".to_string(), json!(description)),
        ]))
    }

    pub fn id(&self) -> Option<&Value> {
        self.0.get("id")
    }

    pub fn title(&self) -> Option<&str> {
        self.0.get("title").and_then(Value::as_str)
    }

    pub fn widget_count(&self) -> usize {
        self.0
            .get("widgets")
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    }

    pub fn layout_type(&self) -> Option<&Value> {
        self.0.get("layout_type")
    }

    pub fn apply_patch(&mut self, patch: DashboardPatch) {
        if let Some(title) = patch.title {
            self.0.insert("title".to_string(), json!(title));
        }
        if let Some(widgets) = patch.widgets {
            self.0.insert("widgets".to_string(), json!(widgets));
        }
    }

    pub fn into_update_payload(mut self) -> Self {
        for field in ["id", "author_handle", "created_at", "modified_at", "url"] {
            self.0.remove(field);
        }
        self
    }
}

/// Lossless Synthetics test representation used for read-modify-write operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SyntheticsTestDocument(Map<String, Value>);

#[derive(Debug, Default)]
pub struct SyntheticsTestPatch {
    pub name: Option<String>,
    pub url: Option<String>,
    pub locations: Option<Vec<String>>,
    pub message: Option<String>,
    pub tags: Option<Vec<String>>,
    pub tick_every: Option<i32>,
}

impl SyntheticsTestDocument {
    pub fn public_id(&self) -> Option<&Value> {
        self.0.get("public_id")
    }

    pub fn name(&self) -> Option<&str> {
        self.0.get("name").and_then(Value::as_str)
    }

    pub fn status(&self) -> Option<&Value> {
        self.0.get("status")
    }

    pub fn apply_patch(&mut self, patch: SyntheticsTestPatch) -> Result<()> {
        if let Some(name) = patch.name {
            self.0.insert("name".to_string(), json!(name));
        }
        if let Some(locations) = patch.locations {
            self.0.insert("locations".to_string(), json!(locations));
        }
        if let Some(message) = patch.message {
            self.0.insert("message".to_string(), json!(message));
        }
        if let Some(tags) = patch.tags {
            self.0.insert("tags".to_string(), json!(tags));
        }
        if let Some(tick_every) = patch.tick_every {
            self.object_at_mut(&["options"])?
                .insert("tick_every".to_string(), json!(tick_every));
        }
        if let Some(url) = patch.url {
            self.object_at_mut(&["config", "request"])?
                .insert("url".to_string(), json!(url));
        }
        Ok(())
    }

    pub fn into_update_payload(mut self) -> Self {
        for field in ["public_id", "created_at", "modified_at"] {
            self.0.remove(field);
        }
        self
    }

    fn object_at_mut(&mut self, path: &[&str]) -> Result<&mut Map<String, Value>> {
        let mut current = &mut self.0;
        for segment in path {
            current = current
                .get_mut(*segment)
                .and_then(Value::as_object_mut)
                .ok_or_else(|| {
                    Error::InvalidResponse(format!(
                        "Synthetics response has no {} object",
                        path.join(".")
                    ))
                })?;
        }
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_patch_preserves_unknown_fields() {
        let mut document: DashboardDocument = serde_json::from_value(json!({
            "id": "abc",
            "title": "Before",
            "widgets": [],
            "future_field": { "kept": true }
        }))
        .unwrap();
        document.apply_patch(DashboardPatch {
            title: Some("After".to_string()),
            widgets: None,
        });

        let payload = serde_json::to_value(document.into_update_payload()).unwrap();
        assert_eq!(payload["title"], "After");
        assert_eq!(payload["future_field"]["kept"], true);
        assert!(payload.get("id").is_none());
    }

    #[test]
    fn synthetics_patch_updates_nested_fields_losslessly() {
        let mut document: SyntheticsTestDocument = serde_json::from_value(json!({
            "public_id": "test-1",
            "name": "Before",
            "options": { "tick_every": 60, "future_option": true },
            "config": { "request": { "url": "https://old", "method": "GET" } },
            "future_field": 42
        }))
        .unwrap();
        document
            .apply_patch(SyntheticsTestPatch {
                url: Some("https://new".to_string()),
                tick_every: Some(300),
                ..SyntheticsTestPatch::default()
            })
            .unwrap();

        let payload = serde_json::to_value(document.into_update_payload()).unwrap();
        assert_eq!(payload["config"]["request"]["url"], "https://new");
        assert_eq!(payload["config"]["request"]["method"], "GET");
        assert_eq!(payload["options"]["future_option"], true);
        assert_eq!(payload["future_field"], 42);
        assert!(payload.get("public_id").is_none());
    }
}
