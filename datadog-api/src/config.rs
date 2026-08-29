#[cfg(feature = "keyring")]
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

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

/// HTTP client configuration for connection pooling and timeouts.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Maximum idle connections per host in the pool (default: 10)
    pub pool_max_idle_per_host: usize,
    /// Idle connection timeout in seconds (default: 90)
    pub pool_idle_timeout_secs: u64,
    /// Enable TCP keepalive with given interval in seconds (default: Some(60))
    pub tcp_keepalive_secs: Option<u64>,
    /// Overall deadline across all retry attempts (default: 90)
    pub total_timeout_secs: u64,
    /// Maximum decoded response body size in bytes (default: 10 MiB)
    #[serde(default = "default_max_response_bytes")]
    pub max_response_bytes: usize,
}

const fn default_max_response_bytes() -> usize {
    10 * 1024 * 1024
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            pool_max_idle_per_host: 10,
            pool_idle_timeout_secs: 90,
            tcp_keepalive_secs: Some(60),
            total_timeout_secs: 90,
            max_response_bytes: default_max_response_bytes(),
        }
    }
}

/// Datadog API configuration containing credentials and regional settings.
#[derive(Clone, Deserialize)]
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
    /// HTTP client configuration (timeouts, connection pool)
    #[serde(default)]
    pub http_config: HttpConfig,
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
            .field("http_config", &self.http_config)
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

/// Datadog sites supported by the public API.
pub const SUPPORTED_SITES: &[&str] = &[
    "datadoghq.com",
    "us3.datadoghq.com",
    "us5.datadoghq.com",
    "datadoghq.eu",
    "ap1.datadoghq.com",
    "ap2.datadoghq.com",
    "uk1.datadoghq.com",
    "ddog-gov.com",
    "us2.ddog-gov.com",
];

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
            http_config: HttpConfig::default(),
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

    /// Validate the configured Datadog site before making network requests.
    pub fn validate_site(&self) -> crate::Result<()> {
        if let Some(base_url) = &self.base_url_override {
            let url = reqwest::Url::parse(base_url).map_err(|error| {
                crate::Error::ConfigError(format!("Invalid base URL override: {error}"))
            })?;
            let is_loopback = url.host_str().is_some_and(|host| {
                host.eq_ignore_ascii_case("localhost")
                    || host
                        .parse::<std::net::IpAddr>()
                        .is_ok_and(|address| address.is_loopback())
            });
            if url.scheme() == "https" || (url.scheme() == "http" && is_loopback) {
                return Ok(());
            }
            return Err(crate::Error::ConfigError(
                "Base URL overrides must use HTTPS; HTTP is allowed only for loopback test servers"
                    .to_string(),
            ));
        }

        if SUPPORTED_SITES.contains(&self.site.as_str()) {
            return Ok(());
        }

        Err(crate::Error::ConfigError(format!(
            "Unsupported DD_SITE '{}'. Supported sites: {}",
            self.site,
            SUPPORTED_SITES.join(", ")
        )))
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

        let config = Self {
            api_key: SecretString::new(api_key),
            app_key: SecretString::new(application_key),
            site,
            retry_config: RetryConfig::default(),
            http_config: HttpConfig::default(),
            unstable_operations: default_unstable_operations(),
            base_url_override: None,
        };
        config.validate_site()?;
        Ok(config)
    }

    /// Load credentials using explicit precedence: environment, keyring, then file.
    ///
    /// If either credential environment variable is present, environment loading is
    /// attempted and partial configuration is reported instead of silently falling back.
    pub fn from_env_or_file() -> crate::Result<Self> {
        if std::env::var_os("DD_API_KEY").is_some() || std::env::var_os("DD_APP_KEY").is_some() {
            return Self::from_env();
        }
        #[cfg(feature = "keyring")]
        if let Some(keyring_cfg) = Self::from_keyring_optional()? {
            return Ok(keyring_cfg);
        }
        Self::from_credentials_file()
    }

    /// Load credentials from environment or the credentials file, without consulting keyring.
    ///
    /// This is intended for one-time keyring enrollment commands.
    pub fn from_env_or_credentials_file() -> crate::Result<Self> {
        if std::env::var_os("DD_API_KEY").is_some() || std::env::var_os("DD_APP_KEY").is_some() {
            return Self::from_env();
        }
        Self::from_credentials_file()
    }

    fn from_credentials_file() -> crate::Result<Self> {
        let home = std::env::var("HOME").map_err(|_| {
            crate::Error::ConfigError("HOME not set; cannot read credentials file".to_string())
        })?;
        let path = PathBuf::from(home)
            .join(".datadog-mcp")
            .join("credentials.json");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path)
                .map_err(|e| {
                    crate::Error::ConfigError(format!(
                        "Failed to inspect {}: {}",
                        path.display(),
                        e
                    ))
                })?
                .permissions()
                .mode()
                & 0o777;
            if mode & 0o077 != 0 {
                return Err(crate::Error::ConfigError(format!(
                    "Credentials file {} must not be accessible by group or others (use chmod 600)",
                    path.display()
                )));
            }
        }

        let content = Zeroizing::new(std::fs::read_to_string(&path).map_err(|e| {
            crate::Error::ConfigError(format!("Failed to read {}: {}", path.display(), e))
        })?);
        let file_cfg: FileCredentials = serde_json::from_str(content.as_str()).map_err(|e| {
            crate::Error::ConfigError(format!(
                "Invalid credentials file {}: {}",
                path.display(),
                e
            ))
        })?;
        let config = Self::new(file_cfg.api_key, file_cfg.app_key)
            .with_site(file_cfg.site.unwrap_or_else(default_site));
        config.validate_site()?;
        Ok(config)
    }

    /// Load configuration from the system keyring entry, if present.
    ///
    /// Profile defaults to `DD_PROFILE` or `default`.
    #[cfg(feature = "keyring")]
    pub fn from_keyring() -> crate::Result<Self> {
        Self::from_keyring_optional()?.ok_or_else(|| {
            crate::Error::ConfigError("No credentials found in the system keyring".to_string())
        })
    }

    #[cfg(feature = "keyring")]
    fn from_keyring_optional() -> crate::Result<Option<Self>> {
        let profile = std::env::var("DD_PROFILE").unwrap_or_else(|_| "default".to_string());
        let entry = Entry::new(KEYRING_SERVICE, &profile)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to access keyring: {e}")))?;
        let password = match entry.get_password() {
            Ok(password) => password,
            Err(keyring::Error::NoEntry) => return Ok(None),
            Err(error) => {
                return Err(crate::Error::ConfigError(format!(
                    "Failed to read keyring entry: {error}"
                )))
            }
        };
        let secret = Zeroizing::new(password);
        let creds: FileCredentials = serde_json::from_str(secret.as_str()).map_err(|e| {
            crate::Error::ConfigError(format!("Invalid keyring credentials format: {e}"))
        })?;
        let config = Self::new(creds.api_key, creds.app_key)
            .with_site(creds.site.unwrap_or_else(default_site));
        config.validate_site()?;
        Ok(Some(config))
    }

    /// Store the current configuration in the system keyring entry.
    ///
    /// Profile defaults to `DD_PROFILE` or `default`.
    #[cfg(feature = "keyring")]
    pub fn store_in_keyring(&self) -> crate::Result<()> {
        let profile = std::env::var("DD_PROFILE").unwrap_or_else(|_| "default".to_string());
        let entry = Entry::new(KEYRING_SERVICE, &profile)
            .map_err(|e| crate::Error::ConfigError(format!("Failed to access keyring: {e}")))?;
        let payload = Zeroizing::new(
            serde_json::to_string(&KeyringCredentials {
                api_key: self.api_key.expose(),
                app_key: self.app_key.expose(),
                site: &self.site,
            })
            .map_err(|e| {
                crate::Error::ConfigError(format!("Failed to serialize credentials: {e}"))
            })?,
        );
        entry.set_password(payload.as_str()).map_err(|e| {
            crate::Error::ConfigError(format!("Failed to store keyring entry: {e}"))
        })?;
        Ok(())
    }
}

/// Wrapper for secrets that zeroize on drop and redact debug output.
#[derive(Clone, Deserialize, Zeroize, ZeroizeOnDrop, PartialEq, Eq)]
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

#[derive(Deserialize)]
struct FileCredentials {
    api_key: String,
    app_key: String,
    #[serde(default)]
    site: Option<String>,
}

#[cfg(feature = "keyring")]
#[derive(Serialize)]
struct KeyringCredentials<'a> {
    api_key: &'a str,
    app_key: &'a str,
    site: &'a str,
}

#[cfg(feature = "keyring")]
const KEYRING_SERVICE: &str = "datadog-mcp";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DatadogClient, Error};
    use std::env;
    use std::sync::{Mutex, MutexGuard, PoisonError};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock_env() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
    }

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
    fn test_supported_sites_validate() {
        for site in SUPPORTED_SITES {
            let config = DatadogConfig::new("api".to_string(), "app".to_string())
                .with_site((*site).to_string());
            assert!(
                config.validate_site().is_ok(),
                "site {site} should validate"
            );
        }
    }

    #[test]
    fn test_unknown_site_is_rejected() {
        let config = DatadogConfig::new("api".to_string(), "app".to_string())
            .with_site("attacker.example".to_string());
        assert!(config.validate_site().is_err());
        assert!(DatadogClient::new(config).is_err());
    }

    #[test]
    fn test_base_url_override_rejects_remote_plaintext_http() {
        let config = DatadogConfig::new("api".to_string(), "app".to_string())
            .with_base_url("http://attacker.example".to_string());
        assert!(config.validate_site().is_err());
        assert!(DatadogClient::new(config).is_err());
    }

    #[test]
    fn test_base_url_override_allows_loopback_http() {
        let config = DatadogConfig::new("api".to_string(), "app".to_string())
            .with_base_url("http://127.0.0.1:8080".to_string());
        assert!(config.validate_site().is_ok());
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
    fn test_from_env_success() {
        let _env_guard = lock_env();
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
    fn test_from_env_default_site() {
        let _env_guard = lock_env();
        env::set_var("DD_API_KEY", "env_api_key");
        env::set_var("DD_APP_KEY", "env_app_key");
        env::remove_var("DD_SITE");

        let config = DatadogConfig::from_env().expect("Failed to create config from env");

        assert_eq!(config.site, "datadoghq.com");

        env::remove_var("DD_API_KEY");
        env::remove_var("DD_APP_KEY");
    }

    #[test]
    fn test_from_env_missing_api_key() {
        let _env_guard = lock_env();
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
    fn test_from_env_missing_app_key() {
        let _env_guard = lock_env();
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
    fn test_secret_debug_is_redacted() {
        let secret = SecretString::new("api_key");
        assert_eq!(format!("{secret:?}"), "[REDACTED]");
        assert_eq!(format!("{secret}"), "[REDACTED]");
    }

    #[test]
    fn test_http_config_default() {
        let config = HttpConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.pool_max_idle_per_host, 10);
        assert_eq!(config.pool_idle_timeout_secs, 90);
        assert_eq!(config.tcp_keepalive_secs, Some(60));
        assert_eq!(config.total_timeout_secs, 90);
        assert_eq!(config.max_response_bytes, 10 * 1024 * 1024);
    }

    #[test]
    fn test_http_config_serialization() {
        let config = HttpConfig {
            timeout_secs: 60,
            pool_max_idle_per_host: 20,
            pool_idle_timeout_secs: 120,
            tcp_keepalive_secs: None,
            total_timeout_secs: 180,
            max_response_bytes: 20 * 1024 * 1024,
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: HttpConfig = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(config.timeout_secs, deserialized.timeout_secs);
        assert_eq!(
            config.pool_max_idle_per_host,
            deserialized.pool_max_idle_per_host
        );
        assert_eq!(config.tcp_keepalive_secs, deserialized.tcp_keepalive_secs);
        assert_eq!(config.max_response_bytes, deserialized.max_response_bytes);
    }
}
