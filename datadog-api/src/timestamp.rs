//! Type-safe timestamp wrappers for Datadog API
//!
//! Provides newtype wrappers to prevent mixing up different time units.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Unix timestamp in seconds since epoch.
///
/// Used for most Datadog API endpoints that accept time ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimestampSecs(pub i64);

impl TimestampSecs {
    /// Create a timestamp for the current time.
    #[must_use]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(duration.as_secs() as i64)
    }

    /// Create a timestamp from milliseconds.
    #[must_use]
    pub const fn from_millis(ms: i64) -> Self {
        Self(ms / 1000)
    }

    /// Convert to milliseconds.
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.0 * 1000
    }

    /// Create a timestamp N seconds ago from now.
    #[must_use]
    pub fn seconds_ago(seconds: i64) -> Self {
        Self(Self::now().0 - seconds)
    }

    /// Create a timestamp N minutes ago from now.
    #[must_use]
    pub fn minutes_ago(minutes: i64) -> Self {
        Self::seconds_ago(minutes * 60)
    }

    /// Create a timestamp N hours ago from now.
    #[must_use]
    pub fn hours_ago(hours: i64) -> Self {
        Self::seconds_ago(hours * 3600)
    }

    /// Create a timestamp N days ago from now.
    #[must_use]
    pub fn days_ago(days: i64) -> Self {
        Self::seconds_ago(days * 86400)
    }

    /// Get the raw seconds value.
    #[must_use]
    pub const fn as_secs(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for TimestampSecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for TimestampSecs {
    fn from(secs: i64) -> Self {
        Self(secs)
    }
}

impl From<TimestampSecs> for i64 {
    fn from(ts: TimestampSecs) -> Self {
        ts.0
    }
}

impl From<Duration> for TimestampSecs {
    fn from(duration: Duration) -> Self {
        Self(duration.as_secs() as i64)
    }
}

/// Unix timestamp in milliseconds since epoch.
///
/// Used for high-precision timing in APM traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimestampMillis(pub i64);

impl TimestampMillis {
    /// Create a timestamp for the current time.
    #[must_use]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(duration.as_millis() as i64)
    }

    /// Convert to seconds.
    #[must_use]
    pub const fn as_secs(&self) -> TimestampSecs {
        TimestampSecs(self.0 / 1000)
    }

    /// Get the raw milliseconds value.
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for TimestampMillis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for TimestampMillis {
    fn from(ms: i64) -> Self {
        Self(ms)
    }
}

impl From<TimestampMillis> for i64 {
    fn from(ts: TimestampMillis) -> Self {
        ts.0
    }
}

impl From<TimestampSecs> for TimestampMillis {
    fn from(ts: TimestampSecs) -> Self {
        Self(ts.0 * 1000)
    }
}

/// Unix timestamp in nanoseconds since epoch.
///
/// Used for APM span timing where nanosecond precision is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimestampNanos(pub i64);

impl TimestampNanos {
    /// Create a timestamp for the current time.
    #[must_use]
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(duration.as_nanos() as i64)
    }

    /// Convert to seconds.
    #[must_use]
    pub const fn as_secs(&self) -> TimestampSecs {
        TimestampSecs(self.0 / 1_000_000_000)
    }

    /// Convert to milliseconds.
    #[must_use]
    pub const fn as_millis(&self) -> TimestampMillis {
        TimestampMillis(self.0 / 1_000_000)
    }

    /// Get the raw nanoseconds value.
    #[must_use]
    pub const fn as_nanos(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for TimestampNanos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for TimestampNanos {
    fn from(ns: i64) -> Self {
        Self(ns)
    }
}

impl From<TimestampNanos> for i64 {
    fn from(ts: TimestampNanos) -> Self {
        ts.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_secs_now() {
        let ts = TimestampSecs::now();
        // Should be after year 2020 (1577836800)
        assert!(ts.0 > 1577836800);
    }

    #[test]
    fn test_timestamp_secs_from_millis() {
        let ts = TimestampSecs::from_millis(1234567890123);
        assert_eq!(ts.0, 1234567890);
    }

    #[test]
    fn test_timestamp_secs_as_millis() {
        let ts = TimestampSecs(1234567890);
        assert_eq!(ts.as_millis(), 1234567890000);
    }

    #[test]
    fn test_timestamp_secs_ago() {
        let now = TimestampSecs::now();
        let one_hour_ago = TimestampSecs::hours_ago(1);
        assert_eq!(now.0 - one_hour_ago.0, 3600);
    }

    #[test]
    fn test_timestamp_millis_to_secs() {
        let ms = TimestampMillis(1234567890123);
        assert_eq!(ms.as_secs().0, 1234567890);
    }

    #[test]
    fn test_timestamp_nanos_conversions() {
        let ns = TimestampNanos(1234567890123456789);
        assert_eq!(ns.as_secs().0, 1234567890);
        assert_eq!(ns.as_millis().0, 1234567890123);
    }

    #[test]
    fn test_timestamp_secs_serialization() {
        let ts = TimestampSecs(1234567890);
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, "1234567890");

        let deserialized: TimestampSecs = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, deserialized);
    }

    #[test]
    fn test_timestamp_from_i64() {
        let ts: TimestampSecs = 1234567890i64.into();
        assert_eq!(ts.0, 1234567890);
    }

    #[test]
    fn test_timestamp_into_i64() {
        let ts = TimestampSecs(1234567890);
        let value: i64 = ts.into();
        assert_eq!(value, 1234567890);
    }

    #[test]
    fn test_timestamp_display() {
        let ts = TimestampSecs(1234567890);
        assert_eq!(format!("{ts}"), "1234567890");
    }

    #[test]
    fn test_timestamp_ordering() {
        let earlier = TimestampSecs(1000);
        let later = TimestampSecs(2000);
        assert!(earlier < later);
    }
}
