//! File-based caching for MCP tool responses
//!
//! Stores API responses to disk for inspection and reduces context usage.
//! Supports both JSON and TOON output formats with automatic cleanup.
//!
//! Cache location priority:
//! 1. `DATADOG_MCP_CACHE_DIR` environment variable
//! 2. `$XDG_CACHE_HOME/datadog-mcp/`
//! 3. `~/.cache/datadog-mcp/`
//! 4. `./datadog_cache/` (fallback)

use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use uuid::Uuid;

use crate::output::OutputFormat;

const CACHE_DIR_NAME: &str = "datadog-mcp";
const LEGACY_CACHE_DIR: &str = "datadog_cache";
pub const MAX_CACHE_READ_BYTES: u64 = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheMaintenance {
    pub retention_deleted: usize,
    pub size_deleted: usize,
}

#[derive(Debug)]
struct CacheStoreInner {
    root: PathBuf,
    max_bytes: u64,
    writes_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CacheStore(Arc<CacheStoreInner>);

impl CacheStore {
    pub fn new(root: PathBuf, max_bytes: u64, writes_enabled: bool) -> Self {
        Self(Arc::new(CacheStoreInner {
            root,
            max_bytes,
            writes_enabled,
        }))
    }

    pub fn disabled() -> Self {
        Self::new(default_cache_dir(), 100 * 1024 * 1024, false)
    }

    pub fn root(&self) -> &Path {
        &self.0.root
    }

    pub fn writes_enabled(&self) -> bool {
        self.0.writes_enabled
    }

    pub async fn initialize(&self, retention_hours: u64) -> Result<CacheMaintenance> {
        if !self.writes_enabled() {
            return Ok(CacheMaintenance {
                retention_deleted: 0,
                size_deleted: 0,
            });
        }

        init_cache_in(self.root()).await?;
        let retention_deleted = cleanup_cache_in(self.root(), retention_hours).await?;
        let size_deleted = enforce_cache_size_in(self.root(), self.0.max_bytes).await?;
        Ok(CacheMaintenance {
            retention_deleted,
            size_deleted,
        })
    }

    pub async fn store<T: Serialize>(
        &self,
        data: &T,
        prefix: &str,
        format: OutputFormat,
    ) -> Result<Option<String>> {
        if !self.writes_enabled() {
            return Ok(None);
        }

        init_cache_in(self.root()).await?;
        let filepath = store_data_in(data, prefix, format, self.root()).await?;
        enforce_cache_size_in(self.root(), self.0.max_bytes).await?;
        Ok(Some(filepath))
    }

    pub async fn cleanup(&self, older_than_hours: u64) -> Result<usize> {
        cleanup_cache_in(self.root(), older_than_hours).await
    }

    pub async fn load(&self, filepath: &str) -> Result<serde_json::Value> {
        load_data_in(filepath, self.root()).await
    }
}

fn unix_timestamp() -> Result<i64> {
    Ok(i64::try_from(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    )?)
}

/// Determine a sensible cache directory respecting OS conventions and overrides.
pub fn default_cache_dir() -> PathBuf {
    // Highest priority: explicit override
    if let Ok(dir) = std::env::var("DATADOG_MCP_CACHE_DIR") {
        return PathBuf::from(dir);
    }

    // Unix: XDG cache dir
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return PathBuf::from(xdg).join(CACHE_DIR_NAME);
    }

    // Windows: LOCALAPPDATA / APPDATA
    #[cfg(windows)]
    {
        if let Ok(dir) = std::env::var("LOCALAPPDATA").or_else(|_| std::env::var("APPDATA")) {
            return PathBuf::from(dir).join(CACHE_DIR_NAME);
        }
    }

    // POSIX fallback: ~/.cache
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".cache").join(CACHE_DIR_NAME);
    }

    // Last resort: legacy relative directory in CWD
    PathBuf::from(LEGACY_CACHE_DIR)
}

async fn init_cache_in(dir: impl AsRef<Path>) -> Result<PathBuf> {
    let cache_path = dir.as_ref().to_path_buf();
    fs::create_dir_all(&cache_path).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&cache_path, std::fs::Permissions::from_mode(0o700)).await?;
    }
    Ok(cache_path)
}

async fn store_data_in<T: Serialize>(
    data: &T,
    prefix: &str,
    format: OutputFormat,
    dir: impl AsRef<Path>,
) -> Result<String> {
    let timestamp = unix_timestamp()?;
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let extension = match format {
        OutputFormat::Json => "json",
        #[cfg(feature = "toon")]
        OutputFormat::Toon => "toon",
    };
    let filename = format!("{}_{}_{}.{}", prefix, timestamp, unique_id, extension);

    let cache_path = dir.as_ref().to_path_buf();
    let filepath = cache_path.join(&filename);

    let content = format.format(data)?;
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

async fn cleanup_cache_in(cache_path: &Path, older_than_hours: u64) -> Result<usize> {
    if !cache_path.exists() {
        return Ok(0);
    }

    let retention_seconds =
        i64::try_from(older_than_hours.saturating_mul(3600)).unwrap_or(i64::MAX);
    let cutoff_time = unix_timestamp()?.saturating_sub(retention_seconds);
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
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    if i64::try_from(modified_time).unwrap_or(i64::MAX) < cutoff_time {
                        fs::remove_file(&path).await?;
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    Ok(deleted_count)
}

async fn enforce_cache_size_in(cache_path: &Path, max_bytes: u64) -> Result<usize> {
    if !cache_path.exists() {
        return Ok(0);
    }

    let mut entries = fs::read_dir(cache_path).await?;
    let mut files = Vec::new();
    let mut total_bytes = 0_u64;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let extension = path.extension().and_then(|value| value.to_str());
        if extension != Some("json") && extension != Some("toon") {
            continue;
        }
        let metadata = entry.metadata().await?;
        if metadata.is_file() {
            total_bytes = total_bytes.saturating_add(metadata.len());
            files.push((
                path,
                metadata.len(),
                metadata.modified().unwrap_or(UNIX_EPOCH),
            ));
        }
    }

    files.sort_by_key(|(_, _, modified)| *modified);
    let mut deleted = 0;
    for (path, size, _) in files {
        if total_bytes <= max_bytes {
            break;
        }
        fs::remove_file(path).await?;
        total_bytes = total_bytes.saturating_sub(size);
        deleted += 1;
    }
    Ok(deleted)
}

async fn load_data_in(filepath: &str, cache_dir: impl AsRef<Path>) -> Result<serde_json::Value> {
    let cache_root = fs::canonicalize(cache_dir.as_ref()).await?;
    let path = fs::canonicalize(filepath).await?;
    if !path.starts_with(&cache_root) {
        anyhow::bail!("Cache file must be inside {}", cache_root.to_string_lossy());
    }

    let extension = path.extension().and_then(|value| value.to_str());
    if extension != Some("json") && extension != Some("toon") {
        anyhow::bail!("Only .json and .toon cache files can be loaded");
    }

    let metadata = fs::metadata(&path).await?;
    if !metadata.is_file() {
        anyhow::bail!("Cache path is not a regular file");
    }
    if metadata.len() > MAX_CACHE_READ_BYTES {
        anyhow::bail!(
            "Cache file is {} bytes; maximum readable size is {} bytes",
            metadata.len(),
            MAX_CACHE_READ_BYTES
        );
    }

    let content = fs::read_to_string(&path).await?;

    let data: serde_json::Value = match extension {
        #[cfg(feature = "toon")]
        Some("toon") => {
            // TOON format is output-only; cached .toon files contain raw TOON text
            // Return as a JSON string value for display purposes
            serde_json::Value::String(content)
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

        let loaded = load_data_in(&filepath, temp_dir.path()).await.unwrap();
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

    #[cfg(feature = "toon")]
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

        // Verify JSON can be loaded and parsed
        let loaded_json = load_data_in(&json_path, temp_dir.path()).await.unwrap();
        assert_eq!(loaded_json, test_data);

        // TOON format is output-only, so loading returns raw text as a string
        let loaded_toon = load_data_in(&toon_path, temp_dir.path()).await.unwrap();
        assert!(loaded_toon.is_string()); // TOON returns as raw text
    }

    #[tokio::test]
    async fn test_load_data() {
        let temp_dir = TempDir::new().unwrap();
        init_cache_in(temp_dir.path()).await.unwrap();

        let test_data = json!({"key": "value", "num": 123});
        let filepath = store_data_in(&test_data, "load_test", OutputFormat::Json, temp_dir.path())
            .await
            .unwrap();

        let loaded = load_data_in(&filepath, temp_dir.path()).await.unwrap();
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

    #[tokio::test]
    #[cfg(unix)]
    async fn test_directory_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache");
        init_cache_in(&cache_dir).await.unwrap();
        let mode = std::fs::metadata(cache_dir).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }

    #[tokio::test]
    async fn test_load_rejects_file_outside_cache() {
        let cache = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        init_cache_in(cache.path()).await.unwrap();
        let filepath = store_data_in(
            &json!({"secret": true}),
            "outside",
            OutputFormat::Json,
            outside.path(),
        )
        .await
        .unwrap();

        let error = load_data_in(&filepath, cache.path()).await.unwrap_err();
        assert!(error.to_string().contains("must be inside"));
    }
}
