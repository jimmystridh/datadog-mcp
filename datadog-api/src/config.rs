use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatadogConfig {
    pub api_key: String,
    pub app_key: String,
    #[serde(default = "default_site")]
    pub site: String,
}

fn default_site() -> String {
    "datadoghq.com".to_string()
}

impl DatadogConfig {
    pub fn new(api_key: String, app_key: String) -> Self {
        Self {
            api_key,
            app_key,
            site: default_site(),
        }
    }

    pub fn with_site(mut self, site: String) -> Self {
        self.site = site;
        self
    }

    pub fn base_url(&self) -> String {
        format!("https://api.{}", self.site)
    }

    pub fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("DATADOG_API_KEY")
            .map_err(|_| crate::Error::ConfigError("DATADOG_API_KEY not set".to_string()))?;

        let app_key = std::env::var("DATADOG_APP_KEY")
            .map_err(|_| crate::Error::ConfigError("DATADOG_APP_KEY not set".to_string()))?;

        let site = std::env::var("DATADOG_SITE").unwrap_or_else(|_| default_site());

        Ok(Self {
            api_key,
            app_key,
            site,
        })
    }
}
