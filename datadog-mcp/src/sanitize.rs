//! Input sanitization for user-provided content
//!
//! Provides validation and sanitization for strings that will be sent to
//! external APIs or stored/logged.

#![allow(dead_code)] // Sanitization functions defined for future use

/// Maximum length for user-provided names (monitor names, dashboard titles, etc.)
pub const MAX_NAME_LENGTH: usize = 256;

/// Maximum length for user-provided messages (alert messages, descriptions, etc.)
pub const MAX_MESSAGE_LENGTH: usize = 4096;

/// Maximum length for user-provided queries
pub const MAX_QUERY_LENGTH: usize = 8192;

/// Sanitize a user-provided name (monitor name, dashboard title, etc.)
///
/// - Trims whitespace
/// - Removes control characters except newlines and tabs
/// - Truncates to MAX_NAME_LENGTH
pub fn sanitize_name(input: &str) -> String {
    sanitize_string(input, MAX_NAME_LENGTH)
}

/// Sanitize a user-provided message (alert message, description, etc.)
///
/// - Trims whitespace
/// - Removes control characters except newlines and tabs
/// - Truncates to MAX_MESSAGE_LENGTH
pub fn sanitize_message(input: &str) -> String {
    sanitize_string(input, MAX_MESSAGE_LENGTH)
}

/// Sanitize a user-provided query
///
/// - Trims whitespace
/// - Removes control characters except newlines and tabs
/// - Truncates to MAX_QUERY_LENGTH
pub fn sanitize_query(input: &str) -> String {
    sanitize_string(input, MAX_QUERY_LENGTH)
}

/// Core sanitization function
fn sanitize_string(input: &str, max_length: usize) -> String {
    let cleaned: String = input
        .trim()
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(max_length)
        .collect();

    cleaned
}

/// Sanitize an optional string, returning None if empty after sanitization
pub fn sanitize_optional(input: Option<String>, max_length: usize) -> Option<String> {
    input.map(|s| sanitize_string(&s, max_length)).filter(|s| !s.is_empty())
}

/// Sanitize a vector of tags
///
/// - Sanitizes each tag
/// - Removes empty tags
/// - Limits to 100 tags maximum
pub fn sanitize_tags(tags: Vec<String>) -> Vec<String> {
    const MAX_TAG_LENGTH: usize = 200;
    const MAX_TAGS: usize = 100;

    tags.into_iter()
        .map(|t| sanitize_string(&t, MAX_TAG_LENGTH))
        .filter(|t| !t.is_empty())
        .take(MAX_TAGS)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name_basic() {
        assert_eq!(sanitize_name("Test Monitor"), "Test Monitor");
    }

    #[test]
    fn test_sanitize_name_trims_whitespace() {
        assert_eq!(sanitize_name("  Test Monitor  "), "Test Monitor");
    }

    #[test]
    fn test_sanitize_name_removes_control_chars() {
        assert_eq!(sanitize_name("Test\x00Monitor\x1F"), "TestMonitor");
    }

    #[test]
    fn test_sanitize_name_preserves_newlines_tabs() {
        assert_eq!(sanitize_name("Line1\nLine2\tTabbed"), "Line1\nLine2\tTabbed");
    }

    #[test]
    fn test_sanitize_name_truncates_long_input() {
        let long_name = "a".repeat(500);
        let result = sanitize_name(&long_name);
        assert_eq!(result.len(), MAX_NAME_LENGTH);
    }

    #[test]
    fn test_sanitize_message_respects_length() {
        let long_message = "x".repeat(5000);
        let result = sanitize_message(&long_message);
        assert_eq!(result.len(), MAX_MESSAGE_LENGTH);
    }

    #[test]
    fn test_sanitize_query_respects_length() {
        let long_query = "q".repeat(10000);
        let result = sanitize_query(&long_query);
        assert_eq!(result.len(), MAX_QUERY_LENGTH);
    }

    #[test]
    fn test_sanitize_optional_none() {
        assert_eq!(sanitize_optional(None, 100), None);
    }

    #[test]
    fn test_sanitize_optional_some() {
        assert_eq!(
            sanitize_optional(Some("test".into()), 100),
            Some("test".into())
        );
    }

    #[test]
    fn test_sanitize_optional_empty_becomes_none() {
        assert_eq!(sanitize_optional(Some("   ".into()), 100), None);
    }

    #[test]
    fn test_sanitize_tags() {
        let tags = vec![
            "env:prod".into(),
            "  service:api  ".into(),
            "".into(),
            "valid".into(),
        ];
        let result = sanitize_tags(tags);
        assert_eq!(result, vec!["env:prod", "service:api", "valid"]);
    }

    #[test]
    fn test_sanitize_tags_limits_count() {
        let tags: Vec<String> = (0..150).map(|i| format!("tag:{}", i)).collect();
        let result = sanitize_tags(tags);
        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_sanitize_unicode() {
        assert_eq!(sanitize_name("日本語モニター"), "日本語モニター");
        assert_eq!(sanitize_name("Émoji 🎉 test"), "Émoji 🎉 test");
    }

    #[test]
    fn test_sanitize_empty_string() {
        assert_eq!(sanitize_name(""), "");
        assert_eq!(sanitize_message("   "), "");
    }
}
