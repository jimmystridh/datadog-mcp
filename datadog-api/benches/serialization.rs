//! Benchmarks for serialization performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use datadog_api::{HttpConfig, RetryConfig, TimestampMillis, TimestampNanos, TimestampSecs};

fn timestamp_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamps");

    // Benchmark timestamp creation
    group.bench_function("TimestampSecs::now", |b| {
        b.iter(|| black_box(TimestampSecs::now()))
    });

    group.bench_function("TimestampMillis::now", |b| {
        b.iter(|| black_box(TimestampMillis::now()))
    });

    group.bench_function("TimestampNanos::now", |b| {
        b.iter(|| black_box(TimestampNanos::now()))
    });

    // Benchmark conversions
    let ts_nanos = TimestampNanos::now();
    group.bench_function("TimestampNanos::as_secs", |b| {
        b.iter(|| black_box(ts_nanos.as_secs()))
    });

    group.bench_function("TimestampNanos::as_millis", |b| {
        b.iter(|| black_box(ts_nanos.as_millis()))
    });

    // Benchmark helper constructors
    group.bench_function("TimestampSecs::hours_ago", |b| {
        b.iter(|| black_box(TimestampSecs::hours_ago(24)))
    });

    group.bench_function("TimestampSecs::days_ago", |b| {
        b.iter(|| black_box(TimestampSecs::days_ago(7)))
    });

    group.finish();
}

fn config_serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_serialization");

    // RetryConfig serialization
    let retry_config = RetryConfig::default();
    let retry_json = serde_json::to_string(&retry_config).unwrap();

    group.throughput(Throughput::Bytes(retry_json.len() as u64));

    group.bench_function("RetryConfig::serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&retry_config).unwrap()))
    });

    group.bench_function("RetryConfig::deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<RetryConfig>(&retry_json).unwrap()))
    });

    // HttpConfig serialization
    let http_config = HttpConfig::default();
    let http_json = serde_json::to_string(&http_config).unwrap();

    group.bench_function("HttpConfig::serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&http_config).unwrap()))
    });

    group.bench_function("HttpConfig::deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<HttpConfig>(&http_json).unwrap()))
    });

    group.finish();
}

fn timestamp_serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("timestamp_serialization");

    let ts_secs = TimestampSecs(1234567890);
    let ts_millis = TimestampMillis(1234567890123);
    let ts_nanos = TimestampNanos(1234567890123456789);

    let json_secs = serde_json::to_string(&ts_secs).unwrap();
    let json_millis = serde_json::to_string(&ts_millis).unwrap();
    let json_nanos = serde_json::to_string(&ts_nanos).unwrap();

    group.bench_function("TimestampSecs::serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&ts_secs).unwrap()))
    });

    group.bench_function("TimestampSecs::deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<TimestampSecs>(&json_secs).unwrap()))
    });

    group.bench_function("TimestampMillis::serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&ts_millis).unwrap()))
    });

    group.bench_function("TimestampMillis::deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<TimestampMillis>(&json_millis).unwrap()))
    });

    group.bench_function("TimestampNanos::serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&ts_nanos).unwrap()))
    });

    group.bench_function("TimestampNanos::deserialize", |b| {
        b.iter(|| black_box(serde_json::from_str::<TimestampNanos>(&json_nanos).unwrap()))
    });

    group.finish();
}

criterion_group!(
    benches,
    timestamp_benchmarks,
    config_serialization_benchmarks,
    timestamp_serialization_benchmarks
);
criterion_main!(benches);
