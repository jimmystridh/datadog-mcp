use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

use crate::output::{Formattable, OutputFormat};

const CACHE_DIR: &str = "datadog_cache";

pub async fn init_cache() -> Result<PathBuf> {
    init_cache_in(CACHE_DIR).await
}

pub async fn init_cache_in(dir: impl AsRef<Path>) -> Result<PathBuf> {
    let cache_path = dir.as_ref().to_path_buf();
    fs::create_dir_all(&cache_path).await?;
    Ok(cache_path)
}

pub async fn store_data<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
) -> Result<String> {
    store_data_in(data, prefix, format, CACHE_DIR).await
}

pub async fn store_data_in<T: Serialize + Formattable>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
    dir: impl AsRef<Path>,
) -> Result<String> {
    let timestamp = Utc::now().timestamp();
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let extension = match format {
        OutputFormat::Json => "json",
        OutputFormat::Toon => "toon",
    };
    let filename = format!("{}_{}_{}.{}", prefix, timestamp, unique_id, extension);

    let cache_path = dir.as_ref().to_path_buf();
    let filepath = cache_path.join(&filename);

    let content = data.format(format)?;
    fs::write(&filepath, &content).await?;

    // Set restrictive permissions (0600) on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        fs::set_permissions(&filepath, permissions).await?;
    }

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

        // Clean up both .json and .toon cache files
        let ext = path.extension().and_then(|s| s.to_str());
        if ext == Some("json") || ext == Some("toon") {
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
    let path = PathBuf::from(filepath);

    let data = match path.extension().and_then(|s| s.to_str()) {
        Some("toon") => {
            // Decode TOON format
            let options = toon::Options::default();
            toon::decode_from_str(&content, &options)?
        }
        Some("json") => {
            // JSON format
            serde_json::from_str(&content)?
        }
        _ => {
            // Default to JSON for backwards compatibility
            serde_json::from_str(&content)?
        }
    };

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = init_cache_in(temp_dir.path()).await.unwrap();
        assert!(cache_dir.exists());
        assert!(cache_dir.is_dir());
    }

    #[tokio::test]
    async fn test_store_data() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({
            "test": "value",
            "number": 42,
            "array": [1, 2, 3]
        });

        let filepath = store_data_in(&test_data, "test", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();
        assert!(PathBuf::from(&filepath).exists());
        assert!(filepath.contains("test_"));
        assert!(filepath.ends_with(".json"));

        let loaded = load_data(&filepath).await.unwrap();
        assert_eq!(loaded, test_data);
    }

    #[tokio::test]
    async fn test_store_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let data1 = json!({"id": 1});
        let data2 = json!({"id": 2});

        let filepath1 = store_data_in(&data1, "multi", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();
        let filepath2 = store_data_in(&data2, "multi", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();

        assert_ne!(filepath1, filepath2);
        assert!(PathBuf::from(&filepath1).exists());
        assert!(PathBuf::from(&filepath2).exists());
    }

    #[tokio::test]
    async fn test_cache_filename_format() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({"test": true});
        let filepath = store_data_in(&test_data, "prefix", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();

        let filename = PathBuf::from(&filepath)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(filename.starts_with("prefix_"));
        assert!(filename.ends_with(".json"));

        let parts: Vec<&str> = filename.split('_').collect();
        assert!(parts.len() >= 3);
    }

    #[tokio::test]
    async fn test_store_data_formats() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({"test": "value", "number": 42});

        // Test JSON format
        let json_path = store_data_in(&test_data, "test_json", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();
        assert!(json_path.ends_with(".json"));
        assert!(PathBuf::from(&json_path).exists());

        // Test TOON format
        let toon_path = store_data_in(&test_data, "test_toon", OutputFormat::Toon, temp_dir.path())
            .await
            .unwrap();
        assert!(toon_path.ends_with(".toon"));
        assert!(PathBuf::from(&toon_path).exists());

        // Verify both can be loaded
        let loaded_json = load_data(&json_path).await.unwrap();
        let loaded_toon = load_data(&toon_path).await.unwrap();
        assert_eq!(loaded_json, test_data);
        assert_eq!(loaded_toon, test_data);
    }

    #[tokio::test]
    async fn test_load_data() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({"key": "value", "num": 123});
        let filepath = store_data_in(&test_data, "load_test", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();

        let loaded = load_data(&filepath).await.unwrap();
        assert_eq!(loaded, test_data);
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({"sensitive": "data"});
        let filepath = store_data_in(&test_data, "secret", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();

        let metadata = std::fs::metadata(&filepath).unwrap();
        let mode = metadata.permissions().mode();
        // Check that only owner has read/write (0600 = 0o100600 with file type bits)
        assert_eq!(mode & 0o777, 0o600);
    }
}
