//! Type-safe ID wrappers for Datadog resources
//!
//! These newtypes prevent accidentally mixing up IDs from different resource types.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Macro to define a type-safe ID wrapper
macro_rules! define_id {
    ($name:ident, $inner:ty, $description:expr) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub $inner);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<$inner> for $name {
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }
    };
}

define_id!(MonitorId, i64, "Unique identifier for a Datadog monitor");
define_id!(
    DashboardId,
    String,
    "Unique identifier for a Datadog dashboard"
);
define_id!(
    DowntimeId,
    i64,
    "Unique identifier for a scheduled downtime"
);
define_id!(
    SyntheticsTestId,
    String,
    "Public ID for a Synthetics test"
);
define_id!(IncidentId, String, "Unique identifier for an incident");
define_id!(SloId, String, "Unique identifier for an SLO");
define_id!(NotebookId, i64, "Unique identifier for a notebook");
define_id!(TeamId, String, "Unique identifier for a team");
define_id!(UserId, String, "Unique identifier for a user");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_id() {
        let id = MonitorId(12345);
        assert_eq!(id.0, 12345);
        assert_eq!(format!("{}", id), "12345");

        let id2: MonitorId = 67890i64.into();
        assert_eq!(id2.0, 67890);
    }

    #[test]
    fn test_dashboard_id() {
        let id = DashboardId("abc-123-xyz".to_string());
        assert_eq!(id.0, "abc-123-xyz");
        assert_eq!(format!("{}", id), "abc-123-xyz");

        let id2: DashboardId = "def-456".to_string().into();
        assert_eq!(id2.0, "def-456");
    }

    #[test]
    fn test_downtime_id() {
        let id = DowntimeId(999);
        assert_eq!(id.0, 999);
    }

    #[test]
    fn test_synthetics_test_id() {
        let id = SyntheticsTestId("syn-abc-123".to_string());
        assert_eq!(id.0, "syn-abc-123");
    }

    #[test]
    fn test_serialization() {
        let monitor_id = MonitorId(42);
        let json = serde_json::to_string(&monitor_id).unwrap();
        assert_eq!(json, "42");

        let dashboard_id = DashboardId("dash-1".to_string());
        let json = serde_json::to_string(&dashboard_id).unwrap();
        assert_eq!(json, "\"dash-1\"");
    }

    #[test]
    fn test_deserialization() {
        let monitor_id: MonitorId = serde_json::from_str("42").unwrap();
        assert_eq!(monitor_id.0, 42);

        let dashboard_id: DashboardId = serde_json::from_str("\"dash-1\"").unwrap();
        assert_eq!(dashboard_id.0, "dash-1");
    }

    #[test]
    fn test_equality() {
        let id1 = MonitorId(100);
        let id2 = MonitorId(100);
        let id3 = MonitorId(200);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(MonitorId(1));
        set.insert(MonitorId(2));
        set.insert(MonitorId(1)); // duplicate

        assert_eq!(set.len(), 2);
    }
}
