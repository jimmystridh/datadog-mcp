use serde::{Deserialize, Serialize};

/// Datadog API configuration containing credentials and regional settings.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatadogConfig {
    /// Datadog API key for authentication
    pub api_key: String,
    /// Datadog application key for authentication
    pub app_key: String,
    /// Datadog site/region (defaults to datadoghq.com)
    #[serde(default = "default_site")]
    pub site: String,
}

const fn default_site_const() -> &'static str {
    "datadoghq.com"
}

fn default_site() -> String {
    default_site_const().to_string()
}

impl DatadogConfig {
    /// Creates a new Datadog configuration with the specified credentials.
    ///
    /// Uses the default site (datadoghq.com / US1 region).
    #[must_use]
    pub fn new(api_key: String, application_key: String) -> Self {
        Self {
            api_key,
            app_key: application_key,
            site: default_site(),
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

    /// Returns the full API base URL for the configured Datadog site.
    #[must_use]
    pub fn base_url(&self) -> String {
        format!("https://api.{}", self.site)
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
            api_key,
            app_key: application_key,
            site,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;
    use super::*;
    use std::env;

    #[test]
    fn test_config_new() {
        let config = DatadogConfig::new(
            "test_api_key".to_string(),
            "test_app_key".to_string(),
        );

        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.app_key, "test_app_key");
        assert_eq!(config.site, "datadoghq.com");
    }

    #[test]
    fn test_config_with_site() {
        let config = DatadogConfig::new(
            "test_api_key".to_string(),
            "test_app_key".to_string(),
        )
        .with_site("datadoghq.eu".to_string());

        assert_eq!(config.site, "datadoghq.eu");
    }

    #[test]
    fn test_base_url_us1() {
        let config = DatadogConfig::new(
            "test_api_key".to_string(),
            "test_app_key".to_string(),
        );

        assert_eq!(config.base_url(), "https://api.datadoghq.com");
    }

    #[test]
    fn test_base_url_eu() {
        let config = DatadogConfig::new(
            "test_api_key".to_string(),
            "test_app_key".to_string(),
        )
        .with_site("datadoghq.eu".to_string());

        assert_eq!(config.base_url(), "https://api.datadoghq.eu");
    }

    #[test]
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
        let config = DatadogConfig::new(
            "api_key".to_string(),
            "app_key".to_string(),
        )
        .with_site("datadoghq.eu".to_string());

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: DatadogConfig =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(config.api_key, deserialized.api_key);
        assert_eq!(config.app_key, deserialized.app_key);
        assert_eq!(config.site, deserialized.site);
    }
}
