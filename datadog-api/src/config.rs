#[cfg(feature = "keyring")]
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Retry configuration for API requests.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Datadog API configuration containing credentials and regional settings.
#[derive(Clone, Deserialize, Serialize)]
pub struct DatadogConfig {
    /// Datadog API key for authentication
    pub api_key: SecretString,
    /// Datadog application key for authentication
    pub app_key: SecretString,
    /// Datadog site/region (defaults to datadoghq.com)
    #[serde(default = "default_site")]
    pub site: String,
    /// Retry configuration
    #[serde(default)]
    pub retry_config: RetryConfig,
    /// List of unstable operations that require the DD-OPERATION-UNSTABLE header
    #[serde(default = "default_unstable_operations")]
    pub unstable_operations: Vec<String>,
    /// Override base URL (for testing with mock servers)
    #[serde(skip)]
    base_url_override: Option<String>,
}

impl fmt::Debug for DatadogConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatadogConfig")
            .field("api_key", &"[REDACTED]")
            .field("app_key", &"[REDACTED]")
            .field("site", &self.site)
            .field("retry_config", &self.retry_config)
            .field("unstable_operations", &self.unstable_operations)
            .field(
                "base_url_override",
                &self.base_url_override.as_ref().map(|_| "[SET]"),
            )
            .finish()
    }
}

const fn default_site_const() -> &'static str {
    "datadoghq.com"
}

fn default_site() -> String {
    default_site_const().to_string()
}

fn default_unstable_operations() -> Vec<String> {
    vec!["incidents".to_string()]
}

impl DatadogConfig {
    /// Creates a new Datadog configuration with the specified credentials.
    ///
    /// Uses the default site (datadoghq.com / US1 region).
    #[must_use]
    pub fn new(api_key: String, application_key: String) -> Self {
        Self {
            api_key: SecretString::new(api_key),
            app_key: SecretString::new(application_key),
            site: default_site(),
            retry_config: RetryConfig::default(),
            unstable_operations: default_unstable_operations(),
            base_url_override: None,
        }
    }

    /// Sets the Datadog site/region for this configuration.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = DatadogConfig::new(api_key, app_key)
    ///     .with_site("datadoghq.eu".to_string());
    /// ```
    #[must_use]
    pub fn with_site(mut self, site: String) -> Self {
        self.site = site;
        self
    }

    /// Sets a custom base URL (for testing with mock servers).
    #[must_use]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url_override = Some(base_url);
        self
    }

    /// Returns the full API base URL for the configured Datadog site.
    #[must_use]
    pub fn base_url(&self) -> String {
        self.base_url_override
            .clone()
            .unwrap_or_else(|| format!("https://api.{}", self.site))
    }

    /// Creates a configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `DD_API_KEY` (required): Datadog API key
    /// - `DD_APP_KEY` (required): Datadog application key
    /// - `DD_SITE` (optional): Datadog site, defaults to datadoghq.com
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are not set.
    pub fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("DD_API_KEY")
            .map_err(|_| crate::Error::ConfigError("DD_API_KEY not set".to_string()))?;

        let application_key = std::env::var("DD_APP_KEY")
            .map_err(|_| crate::Error::ConfigError("DD_APP_KEY not set".to_string()))?;

        let site = std::env::var("DD_SITE").unwrap_or_else(|_| default_site());

        Ok(Self {
            api_key: SecretString::new(api_key),
            app_key: SecretString::new(application_key),
            site,
            retry_config: RetryConfig::default(),
            unstable_operations: default_unstable_operations(),
            base_url_override: None,
        })
    }

    /// Attempt to load credentials from ~/.datadog-mcp/credentials.json, falling back to env vars.
    pub fn from_env_or_file() -> crate::Result<Self> {
        if let Ok(file_cfg) = Self::from_credentials_file() {
            return Ok(file_cfg);
        }
        #[cfg(feature = "keyring")]
        if let Ok(keyring_cfg) = Self::from_keyring() {
            return Ok(keyring_cfg);
        }
        Self::from_env()
    }

    fn from_credentials_file() -> crate::Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            crate::Error::ConfigError("HOME not set; cannot read credentials file".to_string())
        })?;
        let path = PathBuf::from(home)
            .join(".datadog-mcp")
            .join("credentials.json");
        let content = std::fs::read_to_string(&path).map_err(|e| {
            crate::Error::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?;
        let file_cfg: FileCredentials = serde_json::from_str(&content).map_err(|e| {
            crate::Error::ConfigError(format!(
                "Invalid credentials file {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(Self::new(file_cfg.api_key, file_cfg.app_key)
            .with_site(file_cfg.site.unwrap_or_else(default_site)))
    }

    /// Load configuration from the system keyring entry, if present.
    ///
    /// Profile defaults to `DD_PROFILE` or `default`.
    #[cfg(feature = "keyring")]
    pub fn from_keyring() -> crate::Result<Self> {
        let profile = std::env::var("DD_PROFILE").unwrap_or_else(|_| "default".to_string());
        let entry = Entry::new(KEYRING_SERVICE, &profile)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to access keyring: {e}")))?;
        let secret = entry
            .get_password()
            .map_err(|e| crate::Error::ConfigError(format!("Failed to read keyring entry: {e}")))?;
        let creds: FileCredentials = serde_json::from_str(&secret).map_err(|e| {
            crate::Error::ConfigError(format!("Invalid keyring credentials format: {e}"))
        })?;
        Ok(Self::new(creds.api_key, creds.app_key)
            .with_site(creds.site.unwrap_or_else(default_site)))
    }

    /// Store the current configuration in the system keyring entry.
    ///
    /// Profile defaults to `DD_PROFILE` or `default`.
    #[cfg(feature = "keyring")]
    pub fn store_in_keyring(&self) -> crate::Result<()> {
        let profile = std::env::var("DD_PROFILE").unwrap_or_else(|_| "default".to_string());
        let entry = Entry::new(KEYRING_SERVICE, &profile)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to access keyring: {e}")))?;
        let payload = serde_json::to_string(&FileCredentials {
            api_key: self.api_key.expose().to_string(),
            app_key: self.app_key.expose().to_string(),
            site: Some(self.site.clone()),
        })
        .map_err(|e| crate::Error::ConfigError(format!("Failed to serialize credentials: {e}")))?;
        entry.set_password(&payload).map_err(|e| {
            crate::Error::ConfigError(format!("Failed to store keyring entry: {e}"))
        })?;
        Ok(())
    }
}

/// Wrapper for secrets that zeroize on drop and redact debug output.
#[derive(Clone, Deserialize, Serialize, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl PartialEq<str> for SecretString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<String> for SecretString {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<&str> for SecretString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FileCredentials {
    api_key: String,
    app_key: String,
    #[serde(default)]
    site: Option<String>,
}

#[cfg(feature = "keyring")]
const KEYRING_SERVICE: &str = "datadog-mcp";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use serial_test::serial;
    use std::env;

    #[test]
    fn test_config_new() {
        let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.app_key, "test_app_key");
        assert_eq!(config.site, "datadoghq.com");
    }

    #[test]
    fn test_config_with_site() {
        let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
            .with_site("datadoghq.eu".to_string());

        assert_eq!(config.site, "datadoghq.eu");
    }

    #[test]
    fn test_base_url_us1() {
        let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string());

        assert_eq!(config.base_url(), "https://api.datadoghq.com");
    }

    #[test]
    fn test_base_url_eu() {
        let config = DatadogConfig::new("test_api_key".to_string(), "test_app_key".to_string())
            .with_site("datadoghq.eu".to_string());

        assert_eq!(config.base_url(), "https://api.datadoghq.eu");
    }

    #[test]
    #[serial]
    fn test_from_env_success() {
        env::set_var("DD_API_KEY", "env_api_key");
        env::set_var("DD_APP_KEY", "env_app_key");
        env::set_var("DD_SITE", "us3.datadoghq.com");

        let config = DatadogConfig::from_env().expect("Failed to create config from env");

        assert_eq!(config.api_key, "env_api_key");
        assert_eq!(config.app_key, "env_app_key");
        assert_eq!(config.site, "us3.datadoghq.com");

        env::remove_var("DD_API_KEY");
        env::remove_var("DD_APP_KEY");
        env::remove_var("DD_SITE");
    }

    #[test]
    #[serial]
    fn test_from_env_default_site() {
        env::set_var("DD_API_KEY", "env_api_key");
        env::set_var("DD_APP_KEY", "env_app_key");
        env::remove_var("DD_SITE");

        let config = DatadogConfig::from_env().expect("Failed to create config from env");

        assert_eq!(config.site, "datadoghq.com");

        env::remove_var("DD_API_KEY");
        env::remove_var("DD_APP_KEY");
    }

    #[test]
    #[serial]
    fn test_from_env_missing_api_key() {
        env::remove_var("DD_API_KEY");
        env::set_var("DD_APP_KEY", "env_app_key");

        let result = DatadogConfig::from_env();

        assert!(result.is_err());
        if let Err(Error::ConfigError(msg)) = result {
            assert!(msg.contains("DD_API_KEY"));
        } else {
            panic!("Expected ConfigError");
        }

        env::remove_var("DD_APP_KEY");
    }

    #[test]
    #[serial]
    fn test_from_env_missing_app_key() {
        env::set_var("DD_API_KEY", "env_api_key");
        env::remove_var("DD_APP_KEY");

        let result = DatadogConfig::from_env();

        assert!(result.is_err());
        if let Err(Error::ConfigError(msg)) = result {
            assert!(msg.contains("DD_APP_KEY"));
        } else {
            panic!("Expected ConfigError");
        }

        env::remove_var("DD_API_KEY");
    }

    #[test]
    fn test_config_serialization() {
        let config = DatadogConfig::new("api_key".to_string(), "app_key".to_string())
            .with_site("datadoghq.eu".to_string());

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: DatadogConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.app_key, deserialized.app_key);
        assert_eq!(config.site, deserialized.site);
    }
}
