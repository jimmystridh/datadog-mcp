//! Output formatting for MCP responses
//!
//! Supports JSON (default) and TOON (Token-Oriented Object Notation) formats.
//! TOON format typically uses 30-60% fewer tokens than JSON, making it ideal
//! for LLM consumption.

use anyhow::Result;
use serde::Serialize;

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Standard JSON format (default for compatibility)
    #[default]
    Json,
    /// TOON format - optimized for LLM token efficiency
    Toon,
}

/// Trait for types that can be formatted in multiple output formats
pub trait Formattable: Serialize {
    /// Format as JSON (pretty-printed)
    fn format_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Format as TOON (token-efficient for LLMs)
    fn format_toon(&self) -> Result<String>
    where
        Self: Sized,
    {
        let options = toon::Options::default();
        Ok(toon::encode_to_string(self, &options)?)
    }

    /// Format using the specified format
    fn format(&self, format: OutputFormat) -> Result<String>
    where
        Self: Sized,
    {
        match format {
            OutputFormat::Json => self.format_json(),
            OutputFormat::Toon => self.format_toon(),
        }
    }
}

/// Blanket implementation for all Serialize types
impl<T: Serialize> Formattable for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        count: i32,
        items: Vec<String>,
    }

    #[test]
    fn test_json_formatting() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let json = data.format_json().unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"test\""));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_toon_formatting() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let toon = data.format_toon().unwrap();
        // TOON format should be more compact than JSON
        let json = data.format_json().unwrap();
        assert!(toon.len() < json.len());
    }

    #[test]
    fn test_format_enum() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let json = data.format(OutputFormat::Json).unwrap();
        let toon = data.format(OutputFormat::Toon).unwrap();

        assert!(json.contains("\"name\""));
        assert!(!toon.is_empty());
        assert!(toon.len() < json.len());
    }
}
