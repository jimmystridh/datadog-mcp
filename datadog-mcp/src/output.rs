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
    #[cfg(feature = "toon")]
    Toon,
}

impl OutputFormat {
    pub fn format<T: Serialize>(self, value: &T) -> Result<String> {
        match self {
            Self::Json => Ok(serde_json::to_string_pretty(value)?),
            #[cfg(feature = "toon")]
            Self::Toon => Ok(toon::encode(&serde_json::to_value(value)?, None)),
        }
    }
}

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

        let json = OutputFormat::Json.format(&data).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"test\""));
        assert!(json.contains("42"));
    }

    #[cfg(feature = "toon")]
    #[test]
    fn test_toon_formatting() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let toon = OutputFormat::Toon.format(&data).unwrap();
        // TOON format should be more compact than JSON
        let json = OutputFormat::Json.format(&data).unwrap();
        assert!(toon.len() < json.len());
    }

    #[cfg(feature = "toon")]
    #[test]
    fn test_format_enum() {
        let data = TestData {
            name: "test".to_string(),
            count: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let json = OutputFormat::Json.format(&data).unwrap();
        let toon = OutputFormat::Toon.format(&data).unwrap();

        assert!(json.contains("\"name\""));
        assert!(!toon.is_empty());
        assert!(toon.len() < json.len());
    }
}
