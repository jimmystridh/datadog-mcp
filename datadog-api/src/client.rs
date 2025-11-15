use crate::{config::DatadogConfig, error::Error, Result};
use reqwest::{header, Client, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{debug, error, trace};

#[derive(Clone)]
pub struct DatadogClient {
    client: Client,
    config: DatadogConfig,
}

impl DatadogClient {
    pub fn new(config: DatadogConfig) -> Result<Self> {
        // Build HTTP client
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(Error::HttpError)?;

        Ok(Self { client, config })
    }

    pub fn config(&self) -> &DatadogConfig {
        &self.config
    }

    fn build_headers(&self) -> Result<header::HeaderMap> {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            "DD-API-KEY",
            header::HeaderValue::from_str(&self.config.api_key)
                .map_err(|e| Error::ConfigError(format!("Invalid API key: {}", e)))?,
        );

        headers.insert(
            "DD-APPLICATION-KEY",
            header::HeaderValue::from_str(&self.config.app_key)
                .map_err(|e| Error::ConfigError(format!("Invalid app key: {}", e)))?,
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(headers)
    }

    fn add_auth_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder> {
        Ok(builder.headers(self.build_headers()?))
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            trace!("Successful response with status: {}", status);
            response.json::<T>().await.map_err(Error::HttpError)
        } else {
            let status_code = status.as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            error!("API error: {} - {}", status_code, error_body);

            Err(Error::ApiError {
                status: status_code,
                message: error_body,
            })
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("GET {}", url);

        let request = self.client.get(&url);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;
        self.handle_response(response).await
    }

    pub async fn get_with_query<T: DeserializeOwned, Q: serde::Serialize>(
        &self,
        endpoint: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("GET {} with query params", url);

        let request = self.client.get(&url).query(query);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;
        self.handle_response(response).await
    }

    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("POST {}", url);

        let request = self.client.post(&url).json(body);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;
        self.handle_response(response).await
    }

    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("PUT {}", url);

        let request = self.client.put(&url).json(body);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;
        self.handle_response(response).await
    }

    pub async fn delete(&self, endpoint: &str) -> Result<()> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("DELETE {}", url);

        let request = self.client.delete(&url);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let status_code = status.as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            error!("API error: {} - {}", status_code, error_body);

            Err(Error::ApiError {
                status: status_code,
                message: error_body,
            })
        }
    }

    pub async fn delete_with_response<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.config.base_url(), endpoint);
        debug!("DELETE {}", url);

        let request = self.client.delete(&url);
        let request = self.add_auth_headers(request)?;

        let response = request.send().await.map_err(Error::HttpError)?;
        self.handle_response(response).await
    }
}
