use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use reqwest::{Client, RequestBuilder};

use match_composer::api;

use super::error::Error;


#[derive(Clone, Debug)]
pub struct MatchComposerClientConfig {
    pub addr: SocketAddr,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub max_retries: u32,
    pub retry_base: Duration,
    pub retry_backoff: bool,
}

impl Default for MatchComposerClientConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 6657)),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(60),
            max_retries: 30,
            retry_base: Duration::from_secs(2),
            retry_backoff: false,
        }
    }
}


#[derive(Clone, Debug)]
pub struct MatchComposerClient {
    client: Client,
    config: MatchComposerClientConfig,

    base_url: OnceLock<String>,
    start_url: OnceLock<String>,
    stop_url: OnceLock<String>,
    restart_url: OnceLock<String>,
    status_url: OnceLock<String>,
}

impl MatchComposerClient {
    pub fn new(config: MatchComposerClientConfig) -> Self {
        let client = Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .expect("failed to build reqwest client");
        Self {
            client,
            config,
            base_url: Default::default(),
            start_url: Default::default(),
            stop_url: Default::default(),
            restart_url: Default::default(),
            status_url: Default::default(),
        }
    }

    pub fn base_url(&self) -> &str {
        self.base_url.get_or_init(|| format!("http://{}", self.config.addr))
    }

    pub fn start_url(&self) -> &str {
        self.start_url.get_or_init(|| format!("{}/start", self.base_url()))
    }

    pub fn stop_url(&self) -> &str {
        self.stop_url.get_or_init(|| format!("{}/stop", self.base_url()))
    }

    pub fn restart_url(&self) -> &str {
        self.restart_url.get_or_init(|| format!("{}/restart", self.base_url()))
    }

    pub fn status_url(&self) -> &str {
        self.status_url.get_or_init(|| format!("{}/status", self.base_url()))
    }

    pub async fn start(&self) -> Result<api::start::PostResponse, Error> {
        Self::retry(
            "start",
            || self.post_start(),
            self.config.max_retries,
            self.config.retry_base,
            self.config.retry_backoff,
        ).await
    }

    pub async fn post_start(&self) -> Result<api::start::PostResponse, Error> {
        let url = self.start_url();

        let body = api::start::PostRequest {
            config: None,
        };
        let resp = self.client
            .post(url)
            .json(&body);

        Self::resolve(resp).await
    }

    pub async fn stop(&self) -> Result<api::stop::PostResponse, Error> {
        Self::retry(
            "stop",
            || self.post_stop(),
            self.config.max_retries,
            self.config.retry_base,
            self.config.retry_backoff,
        ).await
    }

    pub async fn post_stop(&self) -> Result<api::stop::PostResponse, Error> {
        let url = self.stop_url();

        let body = api::stop::PostRequest {};
        let resp = self.client
            .post(url)
            .json(&body);

        Self::resolve(resp).await
    }

    pub async fn restart(&self) -> Result<api::restart::PostResponse, Error> {
        Self::retry(
            "restart",
            || self.post_restart(),
            self.config.max_retries,
            self.config.retry_base,
            self.config.retry_backoff,
        ).await
    }

    pub async fn post_restart(&self) -> Result<api::restart::PostResponse, Error> {
        let url = self.restart_url();

        let body = api::restart::PostRequest {
            config: None,
        };
        let resp = self.client.post(url).json(&body);

        Self::resolve(resp).await
    }

    pub async fn status(&self) -> Result<api::status::GetResponse, Error> {
        Self::retry(
            "status",
            || self.get_status(),
            self.config.max_retries,
            self.config.retry_base,
            self.config.retry_backoff,
        ).await
    }

    pub async fn get_status(&self) -> Result<api::status::GetResponse, Error> {
        let url = self.status_url();

        let resp = self.client.get(url);

        Self::resolve(resp).await
    }

    async fn resolve<T>(req: RequestBuilder) -> Result<T, Error>
    where T: Serialize + for<'de> Deserialize<'de>
    {
        let resp = req.send().await.map_err(Error::Connection)?;
        Self::resolve_response(resp).await
    }

    async fn resolve_response<T>(resp: reqwest::Response) -> Result<T, Error>
    where T: Serialize + for<'de> Deserialize<'de>
    {
        use common::axum::response::Response as ServerResponse;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::RequestFailed {
                status: status.as_u16(),
                body,
            })
        };

        let resp = match resp.json::<ServerResponse>().await {
            Ok(res) => res,
            Err(e) => {
                return Err(Error::ReqwestDesFailed {
                    source: e,
                    model: "CommonResponse",
                })
            },
        };

        let resp = resp.try_into_generic()
            .map_err(|e| Error::SerdeDesFailed {
                source: e,
                model: "GenericResponse",
            })?;

        Ok(resp.payload)
    }

    /// no backoff for now, just fixed delay
    async fn retry<F, Fut, T>(
        tag: &str,
        func: F,
        n_retry: u32,
        base_delay: Duration,
        back_off: bool,
    ) -> Result<T, Error>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        let mut last_err = None;

        for attempt in 0..n_retry {
            if attempt > 0 {
                let delay = if back_off {
                    base_delay * 2u32.pow(attempt - 1)
                } else {
                    base_delay
                };
                
                info!(
                    "[MatchComposerClient] '{tag}' retry {}/{} after {}ms",
                    attempt + 1,
                    n_retry,
                    delay.as_millis()
                );
                tokio::time::sleep(delay).await;
            }

            let fut = func();
            match fut.await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    warn!("[MatchComposerClient] '{tag}' attempt {}/{} failed: {e:?}", attempt + 1, n_retry);
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            Error::RequestFailed {
                status: 0,
                body: "max retries exceeded".into(),
            }
        }))
    }
}
