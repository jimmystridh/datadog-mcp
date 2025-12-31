//! Input validation for Datadog resources
//!
//! Validates monitor types, dashboard layouts, and other resource configurations
//! before sending to the Datadog API.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid monitor type '{0}'. Valid types: {1}")]
    InvalidMonitorType(String, String),

    #[error("Invalid dashboard layout '{0}'. Valid layouts: ordered, free")]
    InvalidDashboardLayout(String),

    #[error("Empty {0} is not allowed")]
    EmptyField(&'static str),

    #[error("{0}")]
    Custom(String),
}

/// Valid Datadog monitor types
pub const VALID_MONITOR_TYPES: &[&str] = &[
    "metric alert",
    "service check",
    "event alert",
    "event-v2 alert",
    "query alert",
    "composite",
    "synthetics alert",
    "log alert",
    "process alert",
    "rum alert",
    "trace-analytics alert",
    "slo alert",
    "audit alert",
    "ci-pipelines alert",
    "ci-tests alert",
    "error-tracking alert",
    "database-monitoring alert",
];

/// Valid dashboard layout types
pub const VALID_DASHBOARD_LAYOUTS: &[&str] = &["ordered", "free"];

/// Validate monitor type
pub fn validate_monitor_type(monitor_type: &str) -> Result<(), ValidationError> {
    let normalized = monitor_type.to_lowercase();
    if VALID_MONITOR_TYPES.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(ValidationError::InvalidMonitorType(
            monitor_type.to_string(),
            VALID_MONITOR_TYPES.join(", "),
        ))
    }
}

/// Validate dashboard layout type
pub fn validate_dashboard_layout(layout: &str) -> Result<(), ValidationError> {
    let normalized = layout.to_lowercase();
    if VALID_DASHBOARD_LAYOUTS.contains(&normalized.as_str()) {
        Ok(())
    } else {
        Err(ValidationError::InvalidDashboardLayout(layout.to_string()))
    }
}

/// Validate monitor query is not empty
pub fn validate_monitor_query(query: &str) -> Result<(), ValidationError> {
    if query.trim().is_empty() {
        Err(ValidationError::EmptyField("query"))
    } else {
        Ok(())
    }
}

/// Validate monitor name is not empty
pub fn validate_monitor_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        Err(ValidationError::EmptyField("name"))
    } else {
        Ok(())
    }
}

/// Validate dashboard title is not empty
pub fn validate_dashboard_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        Err(ValidationError::EmptyField("title"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_monitor_types() {
        assert!(validate_monitor_type("metric alert").is_ok());
        assert!(validate_monitor_type("log alert").is_ok());
        assert!(validate_monitor_type("query alert").is_ok());
        assert!(validate_monitor_type("composite").is_ok());
        assert!(validate_monitor_type("slo alert").is_ok());
    }

    #[test]
    fn test_monitor_type_case_insensitive() {
        assert!(validate_monitor_type("Metric Alert").is_ok());
        assert!(validate_monitor_type("LOG ALERT").is_ok());
        assert!(validate_monitor_type("Query Alert").is_ok());
    }

    #[test]
    fn test_invalid_monitor_type() {
        let result = validate_monitor_type("invalid type");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid monitor type"));
        assert!(err.to_string().contains("metric alert"));
    }

    #[test]
    fn test_valid_dashboard_layouts() {
        assert!(validate_dashboard_layout("ordered").is_ok());
        assert!(validate_dashboard_layout("free").is_ok());
    }

    #[test]
    fn test_dashboard_layout_case_insensitive() {
        assert!(validate_dashboard_layout("Ordered").is_ok());
        assert!(validate_dashboard_layout("FREE").is_ok());
    }

    #[test]
    fn test_invalid_dashboard_layout() {
        let result = validate_dashboard_layout("grid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid dashboard layout"));
    }

    #[test]
    fn test_validate_monitor_query() {
        assert!(validate_monitor_query("avg:system.cpu.user{*}").is_ok());
        assert!(validate_monitor_query("  ").is_err());
        assert!(validate_monitor_query("").is_err());
    }

    #[test]
    fn test_validate_monitor_name() {
        assert!(validate_monitor_name("My Monitor").is_ok());
        assert!(validate_monitor_name("  ").is_err());
        assert!(validate_monitor_name("").is_err());
    }

    #[test]
    fn test_validate_dashboard_title() {
        assert!(validate_dashboard_title("My Dashboard").is_ok());
        assert!(validate_dashboard_title("  ").is_err());
        assert!(validate_dashboard_title("").is_err());
    }
}
