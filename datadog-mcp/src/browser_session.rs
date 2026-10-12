//! Browser-session authentication for Datadog UI-only read endpoints.

use cookie_scoop::{
    get_cookies, to_cookie_header, BrowserName, CookieHeaderOptions, CookieMode, GetCookiesOptions,
};
use futures_util::StreamExt;
use reqwest::{
    header::{self, HeaderValue},
    redirect::Policy,
    Client, StatusCode, Url,
};
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};
use tracing::{debug, warn};
use zeroize::{Zeroize, Zeroizing};

const WORKLOAD_RECOMMENDATIONS_PATH: &str = "/api/ui/orchestration/autoscaling/workloads/eligible";
const COOKIE_CACHE_TTL: Duration = Duration::from_secs(15 * 60);
const COOKIE_EXTRACTION_TIMEOUT_MS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowserSource {
    Chrome,
    Edge,
    Firefox,
    Safari,
}

impl BrowserSource {
    const fn cookie_scoop_name(self) -> BrowserName {
        match self {
            Self::Chrome => BrowserName::Chrome,
            Self::Edge => BrowserName::Edge,
            Self::Firefox => BrowserName::Firefox,
            Self::Safari => BrowserName::Safari,
        }
    }
}

impl std::fmt::Display for BrowserSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chrome => formatter.write_str("chrome"),
            Self::Edge => formatter.write_str("edge"),
            Self::Firefox => formatter.write_str("firefox"),
            Self::Safari => formatter.write_str("safari"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CookieSelection {
    pub browser: BrowserSource,
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkloadRecommendationsQuery {
    pub query_filter: String,
    pub page: u32,
    pub page_size: u32,
    pub sort_by: String,
    pub sort_order: String,
}

#[derive(Debug)]
pub struct BrowserSessionResponse {
    pub data: Value,
    pub cookie_cache_hit: bool,
    pub cookie_warning_count: usize,
}

struct CachedCookieHeader {
    value: Zeroizing<String>,
    expires_at: Instant,
    warning_count: usize,
}

struct ResolvedCookieHeader {
    value: Zeroizing<String>,
    cache_hit: bool,
    warning_count: usize,
}

#[derive(Clone)]
pub struct BrowserSessionService {
    cookies: Arc<Mutex<HashMap<(String, CookieSelection), CachedCookieHeader>>>,
    cookie_ttl: Duration,
}

impl Default for BrowserSessionService {
    fn default() -> Self {
        Self {
            cookies: Arc::new(Mutex::new(HashMap::new())),
            cookie_ttl: COOKIE_CACHE_TTL,
        }
    }
}

impl BrowserSessionService {
    pub async fn get_workload_recommendations(
        &self,
        site: &str,
        timeout: Duration,
        max_response_bytes: usize,
        selection: CookieSelection,
        query: &WorkloadRecommendationsQuery,
        force_cookie_refresh: bool,
    ) -> anyhow::Result<BrowserSessionResponse> {
        let origin = datadog_ui_origin(site)?;
        let client = WorkloadScalingClient::new(&origin, timeout, max_response_bytes)?;
        let cookies = self
            .resolve_cookie_header(&origin, &selection, force_cookie_refresh)
            .await?;

        match client.get_recommendations(&cookies.value, query).await {
            Ok(data) => Ok(BrowserSessionResponse {
                data,
                cookie_cache_hit: cookies.cache_hit,
                cookie_warning_count: cookies.warning_count,
            }),
            Err(WorkloadRequestError::Unauthorized) if cookies.cache_hit => {
                self.invalidate(&origin, &selection).await;
                let refreshed = self
                    .resolve_cookie_header(&origin, &selection, true)
                    .await?;
                let data = client
                    .get_recommendations(&refreshed.value, query)
                    .await
                    .map_err(|error| request_error(error, &origin))?;
                Ok(BrowserSessionResponse {
                    data,
                    cookie_cache_hit: false,
                    cookie_warning_count: refreshed.warning_count,
                })
            }
            Err(error) => Err(request_error(error, &origin)),
        }
    }

    async fn resolve_cookie_header(
        &self,
        origin: &str,
        selection: &CookieSelection,
        force_refresh: bool,
    ) -> anyhow::Result<ResolvedCookieHeader> {
        let key = (origin.to_string(), selection.clone());
        {
            let mut cookies = self.cookies.lock().await;
            if let Some(cached) = cookies
                .get(&key)
                .filter(|cached| !force_refresh && cached.expires_at > Instant::now())
            {
                return Ok(ResolvedCookieHeader {
                    value: cached.value.clone(),
                    cache_hit: true,
                    warning_count: cached.warning_count,
                });
            }
            cookies.remove(&key);
        }

        let options = cookie_options(origin, selection)?;
        let mut result = get_cookies(options).await;
        let warning_count = result.warnings.len();
        if warning_count > 0 {
            warn!(
                browser = %selection.browser,
                warning_count,
                "Browser cookie extraction completed with warnings"
            );
        }
        if result.cookies.is_empty() {
            anyhow::bail!(
                "no Datadog browser cookies were found for {origin} in {}{}; sign in to Datadog in that browser and select the correct profile",
                selection.browser,
                selection
                    .profile
                    .as_deref()
                    .map(|profile| format!(" profile '{profile}'"))
                    .unwrap_or_default()
            );
        }

        let value = Zeroizing::new(to_cookie_header(
            &result.cookies,
            &CookieHeaderOptions::default(),
        ));
        for cookie in &mut result.cookies {
            cookie.value.zeroize();
        }
        if value.is_empty() {
            anyhow::bail!("Datadog browser cookies were found but could not form a request header");
        }
        HeaderValue::from_str(&value).map_err(|_| {
            anyhow::anyhow!("Datadog browser cookies contain an invalid header value")
        })?;

        self.cookies.lock().await.insert(
            key,
            CachedCookieHeader {
                value: value.clone(),
                expires_at: Instant::now() + self.cookie_ttl,
                warning_count,
            },
        );
        debug!(browser = %selection.browser, "Cached Datadog browser cookies in memory");

        Ok(ResolvedCookieHeader {
            value,
            cache_hit: false,
            warning_count,
        })
    }

    async fn invalidate(&self, origin: &str, selection: &CookieSelection) {
        self.cookies
            .lock()
            .await
            .remove(&(origin.to_string(), selection.clone()));
    }
}

fn cookie_options(origin: &str, selection: &CookieSelection) -> anyhow::Result<GetCookiesOptions> {
    let mut options = GetCookiesOptions::new(origin)
        .browsers(vec![selection.browser.cookie_scoop_name()])
        .mode(CookieMode::First)
        .timeout_ms(COOKIE_EXTRACTION_TIMEOUT_MS);

    if let Some(profile) = selection.profile.as_deref() {
        validate_profile_name(profile)?;
        options = match selection.browser {
            BrowserSource::Chrome => options.chrome_profile(profile),
            BrowserSource::Edge => options.edge_profile(profile),
            BrowserSource::Firefox => options.firefox_profile(profile),
            BrowserSource::Safari => {
                anyhow::bail!("Safari does not support named browser profiles")
            }
        };
    }

    Ok(options)
}

fn validate_profile_name(profile: &str) -> anyhow::Result<()> {
    let is_valid = !profile.is_empty()
        && profile.len() <= 128
        && !profile.contains(['/', '\\'])
        && profile.chars().all(|character| !character.is_control());
    if !is_valid {
        anyhow::bail!(
            "browser_profile must be a profile directory name such as 'Default' or 'Profile 1', not a filesystem path"
        );
    }
    Ok(())
}

pub fn datadog_ui_origin(site: &str) -> anyhow::Result<String> {
    let origin = match site {
        "datadoghq.com" => "https://app.datadoghq.com",
        "datadoghq.eu" => "https://app.datadoghq.eu",
        "us3.datadoghq.com" => "https://us3.datadoghq.com",
        "us5.datadoghq.com" => "https://us5.datadoghq.com",
        "ap1.datadoghq.com" => "https://ap1.datadoghq.com",
        "ap2.datadoghq.com" => "https://ap2.datadoghq.com",
        "uk1.datadoghq.com" => "https://uk1.datadoghq.com",
        "ddog-gov.com" => "https://app.ddog-gov.com",
        "us2.ddog-gov.com" => "https://us2.ddog-gov.com",
        _ => anyhow::bail!("unsupported Datadog site for browser-session access: {site}"),
    };
    Ok(origin.to_string())
}

struct WorkloadScalingClient {
    client: Client,
    endpoint: Url,
    max_response_bytes: usize,
}

impl WorkloadScalingClient {
    fn new(origin: &str, timeout: Duration, max_response_bytes: usize) -> anyhow::Result<Self> {
        let endpoint = Url::parse(origin)?.join(WORKLOAD_RECOMMENDATIONS_PATH)?;
        let client = Client::builder()
            .timeout(timeout)
            .redirect(Policy::none())
            .user_agent(concat!("datadog-mcp/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            client,
            endpoint,
            max_response_bytes,
        })
    }

    async fn get_recommendations(
        &self,
        cookie_header: &str,
        query: &WorkloadRecommendationsQuery,
    ) -> Result<Value, WorkloadRequestError> {
        let mut cookies = HeaderValue::from_str(cookie_header)
            .map_err(|_| WorkloadRequestError::InvalidCookieHeader)?;
        cookies.set_sensitive(true);
        let response = self
            .client
            .get(self.endpoint.clone())
            .query(query)
            .header(header::ACCEPT, "application/json")
            .header(header::COOKIE, cookies)
            .send()
            .await?;

        match response.status() {
            StatusCode::UNAUTHORIZED => return Err(WorkloadRequestError::Unauthorized),
            StatusCode::FORBIDDEN => return Err(WorkloadRequestError::Forbidden),
            status if !status.is_success() => {
                return Err(WorkloadRequestError::UnexpectedStatus(status.as_u16()))
            }
            _ => {}
        }

        if let Some(length) = response.content_length() {
            let size = usize::try_from(length).unwrap_or(usize::MAX);
            if size > self.max_response_bytes {
                return Err(WorkloadRequestError::ResponseTooLarge {
                    size,
                    limit: self.max_response_bytes,
                });
            }
        }

        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let size = body.len().saturating_add(chunk.len());
            if size > self.max_response_bytes {
                return Err(WorkloadRequestError::ResponseTooLarge {
                    size,
                    limit: self.max_response_bytes,
                });
            }
            body.extend_from_slice(&chunk);
        }

        Ok(serde_json::from_slice(&body)?)
    }
}

#[derive(Debug, thiserror::Error)]
enum WorkloadRequestError {
    #[error("browser session is unauthorized")]
    Unauthorized,
    #[error("browser session lacks workload-scaling access")]
    Forbidden,
    #[error("Datadog UI returned HTTP {0}")]
    UnexpectedStatus(u16),
    #[error("browser cookies contain an invalid HTTP header value")]
    InvalidCookieHeader,
    #[error("Datadog UI response exceeded the {limit}-byte limit after {size} bytes")]
    ResponseTooLarge { size: usize, limit: usize },
    #[error("browser-session HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Datadog UI returned invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

fn request_error(error: WorkloadRequestError, origin: &str) -> anyhow::Error {
    match error {
        WorkloadRequestError::Unauthorized => anyhow::anyhow!(
            "Datadog rejected the local browser session for {origin}; sign in again in the selected browser profile"
        ),
        WorkloadRequestError::Forbidden => anyhow::anyhow!(
            "the selected Datadog browser session lacks permission to read workload-scaling recommendations"
        ),
        error => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn maps_every_supported_datadog_site_to_its_ui_origin() {
        let cases = [
            ("datadoghq.com", "https://app.datadoghq.com"),
            ("datadoghq.eu", "https://app.datadoghq.eu"),
            ("us3.datadoghq.com", "https://us3.datadoghq.com"),
            ("us5.datadoghq.com", "https://us5.datadoghq.com"),
            ("ap1.datadoghq.com", "https://ap1.datadoghq.com"),
            ("ap2.datadoghq.com", "https://ap2.datadoghq.com"),
            ("uk1.datadoghq.com", "https://uk1.datadoghq.com"),
            ("ddog-gov.com", "https://app.ddog-gov.com"),
            ("us2.ddog-gov.com", "https://us2.ddog-gov.com"),
        ];
        for (site, expected) in cases {
            assert_eq!(datadog_ui_origin(site).unwrap(), expected);
        }
        assert!(datadog_ui_origin("example.com").is_err());
    }

    #[test]
    fn builds_cookie_options_for_one_explicit_profile() {
        let selection = CookieSelection {
            browser: BrowserSource::Chrome,
            profile: Some("Profile 1".to_string()),
        };

        let options = cookie_options("https://app.datadoghq.eu", &selection).unwrap();

        assert_eq!(options.browsers, Some(vec![BrowserName::Chrome]));
        assert_eq!(options.chrome_profile.as_deref(), Some("Profile 1"));
        assert_eq!(options.mode, Some(CookieMode::First));
        assert_eq!(options.timeout_ms, Some(COOKIE_EXTRACTION_TIMEOUT_MS));
    }

    #[test]
    fn browser_profile_cannot_select_an_arbitrary_path() {
        for profile in ["../Profile 1", "/tmp/Cookies", "Profile\\Cookies", ""] {
            assert!(validate_profile_name(profile).is_err());
        }
    }

    fn query() -> WorkloadRecommendationsQuery {
        WorkloadRecommendationsQuery {
            query_filter: "kube_cluster_name:staging".to_string(),
            page: 2,
            page_size: 25,
            sort_by: "estimated_savings".to_string(),
            sort_order: "desc".to_string(),
        }
    }

    #[tokio::test]
    async fn workload_request_sends_browser_cookie_and_expected_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(WORKLOAD_RECOMMENDATIONS_PATH))
            .and(header("cookie", "session=fake-secret"))
            .and(query_param("query_filter", "kube_cluster_name:staging"))
            .and(query_param("page", "2"))
            .and(query_param("page_size", "25"))
            .and(query_param("sort_by", "estimated_savings"))
            .and(query_param("sort_order", "desc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total_count": 1,
                "workloads": [{"name": "api"}]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client =
            WorkloadScalingClient::new(&server.uri(), Duration::from_secs(5), 1024 * 1024).unwrap();

        let result = client
            .get_recommendations("session=fake-secret", &query())
            .await
            .unwrap();

        assert_eq!(result["total_count"], 1);
    }

    #[tokio::test]
    async fn workload_request_does_not_follow_redirects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(WORKLOAD_RECOMMENDATIONS_PATH))
            .respond_with(
                ResponseTemplate::new(302).insert_header("location", "https://example.com/"),
            )
            .mount(&server)
            .await;
        let client =
            WorkloadScalingClient::new(&server.uri(), Duration::from_secs(5), 1024 * 1024).unwrap();

        let error = client
            .get_recommendations("session=fake-secret", &query())
            .await
            .unwrap_err();

        assert!(matches!(error, WorkloadRequestError::UnexpectedStatus(302)));
        assert!(!error.to_string().contains("fake-secret"));
    }

    #[tokio::test]
    async fn workload_request_reports_unauthorized_without_exposing_cookies() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(WORKLOAD_RECOMMENDATIONS_PATH))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;
        let client =
            WorkloadScalingClient::new(&server.uri(), Duration::from_secs(5), 1024).unwrap();

        let error = client
            .get_recommendations("session=fake-secret", &query())
            .await
            .unwrap_err();

        assert!(matches!(error, WorkloadRequestError::Unauthorized));
        assert!(!error.to_string().contains("fake-secret"));
    }

    #[tokio::test]
    async fn workload_request_rejects_oversized_responses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(WORKLOAD_RECOMMENDATIONS_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_string("x".repeat(1025)))
            .mount(&server)
            .await;
        let client =
            WorkloadScalingClient::new(&server.uri(), Duration::from_secs(5), 1024).unwrap();

        let error = client
            .get_recommendations("session=fake-secret", &query())
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            WorkloadRequestError::ResponseTooLarge { .. }
        ));
    }
}
