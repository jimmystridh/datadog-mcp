//! Property-based tests for serialization roundtrips
//!
//! Uses proptest to verify that serialization followed by deserialization
//! produces equivalent values for various types.

use datadog_api::{
    CacheInfo, HttpConfig, PaginationMeta, RateLimitConfig, RetryConfig, TimestampMillis,
    TimestampNanos, TimestampSecs,
};
use proptest::prelude::*;

// Strategy for generating valid timestamps (post-2000)
fn timestamp_secs_strategy() -> impl Strategy<Value = TimestampSecs> {
    (946684800i64..2000000000i64).prop_map(TimestampSecs)
}

fn timestamp_millis_strategy() -> impl Strategy<Value = TimestampMillis> {
    (946684800000i64..2000000000000i64).prop_map(TimestampMillis)
}

fn timestamp_nanos_strategy() -> impl Strategy<Value = TimestampNanos> {
    (946684800000000000i64..2000000000000000000i64).prop_map(TimestampNanos)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // Timestamp roundtrips
    #[test]
    fn timestamp_secs_roundtrip(ts in timestamp_secs_strategy()) {
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: TimestampSecs = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(ts.as_secs(), parsed.as_secs());
    }

    #[test]
    fn timestamp_millis_roundtrip(ts in timestamp_millis_strategy()) {
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: TimestampMillis = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(ts.as_millis(), parsed.as_millis());
    }

    #[test]
    fn timestamp_nanos_roundtrip(ts in timestamp_nanos_strategy()) {
        let json = serde_json::to_string(&ts).unwrap();
        let parsed: TimestampNanos = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(ts.as_nanos(), parsed.as_nanos());
    }

    // Config roundtrips
    #[test]
    fn retry_config_roundtrip(
        max_retries in 0u32..10,
        initial_backoff in 10u64..1000,
        max_backoff in 1000u64..60000,
        multiplier in 1.0f64..5.0
    ) {
        let config = RetryConfig {
            max_retries,
            initial_backoff_ms: initial_backoff,
            max_backoff_ms: max_backoff,
            backoff_multiplier: multiplier,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: RetryConfig = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(config.max_retries, parsed.max_retries);
        prop_assert_eq!(config.initial_backoff_ms, parsed.initial_backoff_ms);
        prop_assert_eq!(config.max_backoff_ms, parsed.max_backoff_ms);
        // Float comparison with tolerance
        prop_assert!((config.backoff_multiplier - parsed.backoff_multiplier).abs() < 0.0001);
    }

    #[test]
    fn http_config_roundtrip(
        timeout in 1u64..300,
        pool_size in 1usize..100,
        idle_timeout in 10u64..600,
        keepalive in prop::option::of(10u64..300)
    ) {
        let config = HttpConfig {
            timeout_secs: timeout,
            pool_max_idle_per_host: pool_size,
            pool_idle_timeout_secs: idle_timeout,
            tcp_keepalive_secs: keepalive,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: HttpConfig = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(config.timeout_secs, parsed.timeout_secs);
        prop_assert_eq!(config.pool_max_idle_per_host, parsed.pool_max_idle_per_host);
        prop_assert_eq!(config.pool_idle_timeout_secs, parsed.pool_idle_timeout_secs);
        prop_assert_eq!(config.tcp_keepalive_secs, parsed.tcp_keepalive_secs);
    }

    // Cache info validator property
    #[test]
    fn cache_info_has_validators_property(
        etag in prop::option::of("\"[a-zA-Z0-9]{1,32}\""),
        last_modified in prop::option::of("[A-Za-z]{3}, [0-9]{2} [A-Za-z]{3} [0-9]{4}")
    ) {
        let info = CacheInfo { etag: etag.clone(), last_modified: last_modified.clone() };

        // has_validators should be true if either field is Some
        let expected = etag.is_some() || last_modified.is_some();
        prop_assert_eq!(info.has_validators(), expected);
    }

    // Timestamp conversion properties
    #[test]
    fn timestamp_millis_to_secs_preserves_value(ms in 946684800000i64..2000000000000i64) {
        let ts_millis = TimestampMillis(ms);
        let ts_secs = ts_millis.as_secs();
        prop_assert_eq!(ts_secs.as_secs(), ms / 1000);
    }

    #[test]
    fn timestamp_secs_to_millis_multiplies(secs in 946684800i64..2000000000i64) {
        let ts = TimestampSecs(secs);
        prop_assert_eq!(ts.as_millis(), secs * 1000);
    }

    #[test]
    fn timestamp_nanos_to_secs_divides(nanos in 946684800000000000i64..2000000000000000000i64) {
        let ts = TimestampNanos(nanos);
        prop_assert_eq!(ts.as_secs().as_secs(), nanos / 1_000_000_000);
    }

    #[test]
    fn timestamp_nanos_to_millis_divides(nanos in 946684800000000000i64..2000000000000000000i64) {
        let ts = TimestampNanos(nanos);
        prop_assert_eq!(ts.as_millis().as_millis(), nanos / 1_000_000);
    }

    // Pagination meta properties
    #[test]
    fn pagination_meta_has_next_with_offset(offset in prop::option::of(0i64..10000)) {
        let meta = PaginationMeta {
            total_count: None,
            total_pages: None,
            next_offset: offset,
            next_cursor: None,
        };
        prop_assert_eq!(meta.has_next(), offset.is_some());
    }

    #[test]
    fn pagination_meta_has_next_with_cursor(cursor in prop::option::of("[a-z]{1,10}")) {
        let meta = PaginationMeta {
            total_count: None,
            total_pages: None,
            next_offset: None,
            next_cursor: cursor.clone(),
        };
        prop_assert_eq!(meta.has_next(), cursor.is_some());
    }

    // Timestamp ordering property
    #[test]
    fn timestamp_ordering_consistent(a in 0i64..1000000000, b in 0i64..1000000000) {
        let ts_a = TimestampSecs(a);
        let ts_b = TimestampSecs(b);

        if a < b {
            prop_assert!(ts_a < ts_b);
        } else if a > b {
            prop_assert!(ts_a > ts_b);
        } else {
            prop_assert_eq!(ts_a, ts_b);
        }
    }

    // From/Into roundtrip
    #[test]
    fn timestamp_from_into_roundtrip(secs in 0i64..2000000000) {
        let ts: TimestampSecs = secs.into();
        let back: i64 = ts.into();
        prop_assert_eq!(secs, back);
    }
}

// Non-proptest property tests for edge cases
#[test]
fn timestamp_secs_from_millis_truncates() {
    let ts = TimestampSecs::from_millis(1234567890999);
    assert_eq!(ts.as_secs(), 1234567890);
}

#[test]
fn timestamp_ordering_works() {
    let earlier = TimestampSecs(1000);
    let later = TimestampSecs(2000);
    assert!(earlier < later);
    assert!(later > earlier);
    assert_eq!(TimestampSecs(1000), TimestampSecs(1000));
}

#[test]
fn rate_limit_config_disabled_constructor() {
    let config = RateLimitConfig::disabled();
    assert!(!config.enabled);
}
