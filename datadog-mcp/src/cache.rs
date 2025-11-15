use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

const CACHE_DIR: &str = "datadog_cache";

pub async fn init_cache() -> Result<PathBuf> {
    let cache_path = PathBuf::from(CACHE_DIR);
    fs::create_dir_all(&cache_path).await?;
    Ok(cache_path)
}

pub async fn store_data<T: Serialize>(data: &T, prefix: &str) -> Result<String> {
    let timestamp = Utc::now().timestamp();
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let filename = format!("{}_{}_{}.json", prefix, timestamp, unique_id);

    let cache_path = PathBuf::from(CACHE_DIR);
    let filepath = cache_path.join(&filename);

    let json = serde_json::to_string_pretty(data)?;
    fs::write(&filepath, json).await?;

    Ok(filepath.to_string_lossy().to_string())
}

pub async fn cleanup_cache(older_than_hours: u64) -> Result<usize> {
    let cache_path = PathBuf::from(CACHE_DIR);

    if !cache_path.exists() {
        return Ok(0);
    }

    let cutoff_time = Utc::now().timestamp() - (older_than_hours as i64 * 3600);
    let mut deleted_count = 0;

    let mut entries = fs::read_dir(&cache_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(metadata) = fs::metadata(&path).await {
                if let Ok(modified) = metadata.modified() {
                    let modified_time = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;

                    if modified_time < cutoff_time {
                        fs::remove_file(&path).await?;
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

pub async fn load_data(filepath: &str) -> Result<serde_json::Value> {
    let content = fs::read_to_string(filepath).await?;
    let data = serde_json::from_str(&content)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::Path;
    use tokio::fs;

    #[tokio::test]
    async fn test_init_cache() {
        let cache_dir = init_cache().await.unwrap();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[tokio::test]
    async fn test_store_data() {
        let test_data = json!({
            "test": "value",
            "number": 42,
            "array": [1, 2, 3]
        });

        let filepath = store_data(&test_data, "test").await.unwrap();
        assert!(Path::new(&filepath).exists());
        assert!(filepath.contains("test_"));

        let content = fs::read_to_string(&filepath).await.unwrap();
        let loaded: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded, test_data);

        let _ = fs::remove_file(&filepath).await;
    }

    #[tokio::test]
    async fn test_store_multiple_files() {
        let data1 = json!({"id": 1});
        let data2 = json!({"id": 2});

        let filepath1 = store_data(&data1, "multi").await.unwrap();
        let filepath2 = store_data(&data2, "multi").await.unwrap();

        assert_ne!(filepath1, filepath2);
        assert!(Path::new(&filepath1).exists());
        assert!(Path::new(&filepath2).exists());

        let _ = fs::remove_file(&filepath1).await;
        let _ = fs::remove_file(&filepath2).await;
    }

    #[tokio::test]
    async fn test_cache_filename_format() {
        let test_data = json!({"test": true});
        let filepath = store_data(&test_data, "prefix").await.unwrap();

        let filename = Path::new(&filepath).file_name().unwrap().to_str().unwrap();
        assert!(filename.starts_with("prefix_"));
        assert!(filename.ends_with(".json"));

        let parts: Vec<&str> = filename.split('_').collect();
        assert!(parts.len() >= 3);

        let _ = fs::remove_file(&filepath).await;
    }

    #[tokio::test]
    async fn test_load_data() {
        let _ = init_cache().await;
        let test_data = json!({"key": "value", "num": 123});
        let filepath = store_data(&test_data, "load_test").await.unwrap();

        let loaded = load_data(&filepath).await.unwrap();
        assert_eq!(loaded, test_data);

        let _ = fs::remove_file(&filepath).await;
    }
}
