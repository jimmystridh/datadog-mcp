//! Cache analysis and management tools

use crate::cache::{cleanup_cache, load_data};
use crate::response::tool_error;
use serde_json::{json, Value};
use tracing::info;

pub async fn analyze_data(
    filepath: String,
    analysis_type: Option<String>,
) -> anyhow::Result<Value> {
    info!("Analyzing data from: {}", filepath);

    let data = load_data(&filepath).await?;
    let analysis = analysis_type.unwrap_or_else(|| "summary".to_string());

    let result = match analysis.as_str() {
        "summary" => generate_summary(&data),
        "stats" => calculate_stats(&data),
        "trends" => analyze_trends(&data),
        _ => {
            return Ok(json!({
                "error": format!("Unknown analysis type: {}", analysis),
                "status": "error",
            }));
        }
    };

    info!("Completed {} analysis of {}", analysis, filepath);

    Ok(json!({
        "analysis_type": analysis,
        "filepath": filepath,
        "result": result,
        "status": "success",
    }))
}

pub async fn cleanup_cache_tool(older_than_hours: Option<u64>) -> anyhow::Result<Value> {
    let hours = older_than_hours.unwrap_or(24);
    info!("Cleaning up cache files older than {} hours", hours);

    match cleanup_cache(hours).await {
        Ok(deleted_count) => {
            info!(
                "Cleaned up {} files older than {} hours",
                deleted_count, hours
            );
            Ok(json!({
                "summary": format!("Cleaned up {} files older than {} hours", deleted_count, hours),
                "deleted_count": deleted_count,
                "cache_directory": "datadog_cache",
                "status": "success",
            }))
        }
        Err(e) => Ok(tool_error("Failed to cleanup cache", e)),
    }
}

fn generate_summary(data: &Value) -> Value {
    let mut summary = json!({
        "data_type": "unknown",
        "record_count": 0,
        "key_insights": []
    });

    if let Some(obj) = data.as_object() {
        if obj.contains_key("series") {
            if let Some(series) = obj["series"].as_array() {
                summary["data_type"] = json!("metrics");
                summary["record_count"] = json!(series.len());

                let total_points: usize = series
                    .iter()
                    .filter_map(|s| s["pointlist"].as_array())
                    .map(|p| p.len())
                    .sum();

                summary["total_data_points"] = json!(total_points);

                if total_points > 1000 {
                    if let Some(insights) = summary["key_insights"].as_array_mut() {
                        insights.push(json!("Large dataset - consider aggregation"));
                    }
                }
            }
        } else if let Some(arr) = data.as_array() {
            if !arr.is_empty() {
                if let Some(first) = arr[0].as_object() {
                    if first.contains_key("overall_state") {
                        summary["data_type"] = json!("monitors");
                        summary["record_count"] = json!(arr.len());

                        let alerting = arr
                            .iter()
                            .filter(|m| m["overall_state"].as_str() == Some("Alert"))
                            .count();

                        summary["alerting_monitors"] = json!(alerting);

                        if alerting > 0 {
                            if let Some(insights) = summary["key_insights"].as_array_mut() {
                                insights.push(json!(format!(
                                    "{} monitors currently alerting",
                                    alerting
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    summary
}

fn calculate_stats(data: &Value) -> Value {
    let mut stats = json!({
        "calculated_at": chrono::Utc::now().to_rfc3339()
    });

    if let Some(obj) = data.as_object() {
        if let Some(series) = obj.get("series").and_then(|s| s.as_array()) {
            let mut all_values = Vec::new();

            for s in series {
                if let Some(pointlist) = s["pointlist"].as_array() {
                    for point in pointlist {
                        if let Some(arr) = point.as_array() {
                            if arr.len() >= 2 {
                                if let Some(val) = arr[1].as_f64() {
                                    all_values.push(val);
                                }
                            }
                        }
                    }
                }
            }

            if !all_values.is_empty() {
                let min = all_values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = all_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = all_values.iter().sum();
                let avg = sum / all_values.len() as f64;

                stats["min_value"] = json!(min);
                stats["max_value"] = json!(max);
                stats["avg_value"] = json!(avg);
                stats["total_points"] = json!(all_values.len());
            }
        }
    }

    stats
}

fn analyze_trends(data: &Value) -> Value {
    let mut trends = json!({
        "trend_direction": "stable",
        "analyzed_at": chrono::Utc::now().to_rfc3339()
    });

    if let Some(obj) = data.as_object() {
        if let Some(series) = obj.get("series").and_then(|s| s.as_array()) {
            if let Some(first_series) = series.first() {
                if let Some(pointlist) = first_series["pointlist"].as_array() {
                    let values: Vec<f64> = pointlist
                        .iter()
                        .filter_map(|p| p.as_array())
                        .filter_map(|arr| {
                            if arr.len() >= 2 {
                                arr[1].as_f64()
                            } else {
                                None
                            }
                        })
                        .collect();

                    if values.len() >= 2 {
                        let first_val = values[0];
                        let last_val = values[values.len() - 1];

                        if first_val != 0.0 {
                            let change_pct = ((last_val - first_val) / first_val) * 100.0;
                            trends["change_percentage"] = json!(format!("{:.2}", change_pct));

                            if change_pct > 10.0 {
                                trends["trend_direction"] = json!("increasing");
                            } else if change_pct < -10.0 {
                                trends["trend_direction"] = json!("decreasing");
                            }
                        }
                    }
                }
            }
        }
    }

    trends
}
