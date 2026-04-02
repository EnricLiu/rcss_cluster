use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Duration;
use log::{debug, info, warn};
use reqwest::Client;

use super::error::Error;
use super::response::StatusResponse;

#[derive(Clone, Debug)]
pub struct MatchComposerClientConfig {
    pub addr: SocketAddr,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub start_max_retries: u32,
    pub start_retry_base: Duration,
}

impl Default for MatchComposerClientConfig {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 6657)),
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(60),
            start_max_retries: 3,
            start_retry_base: Duration::from_secs(1),
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

    /// POST /start with built-in retry logic for sidecar startup race.
    /// Retries up to 3 times with exponential backoff (1s, 2s, 4s).
    pub async fn start(&self) -> Result<(), Error> {
        let url = self.start_url();
        
        let n_retry = self.config.start_max_retries;
        
        let mut last_err = None;
        for attempt in 0..n_retry {
            if attempt > 0 {
                let backoff = self.config.start_retry_base * 2u32.pow(attempt - 1);
                info!(
                    "[MatchComposerClient] start retry {}/{} after {}ms",
                    attempt + 1,
                    n_retry,
                    backoff.as_millis()
                );
                tokio::time::sleep(backoff).await;
            }

            match self.client.post(url).header("content-type", "application/json").body("{}").send().await {
                Ok(resp) => {
                    return check_response(resp).await.map(|_| ());
                }
                Err(e) if e.is_connect() => {
                    warn!(
                        "[MatchComposerClient] start attempt {}/{} connection failed: {e}",
                        attempt + 1,
                        n_retry
                    );
                    last_err = Some(Error::Connection(e));
                    continue;
                }
                Err(e) => return Err(Error::Connection(e)),
            }
        }

        Err(last_err.unwrap_or_else(|| {
            Error::RequestFailed {
                status: 0,
                body: "max retries exceeded".into(),
            }
        }))
    }

    /// POST /stop
    pub async fn stop(&self) -> Result<(), Error> {
        let url = self.stop_url();
        let resp = self.client.post(url).send().await.map_err(Error::Connection)?;
        check_response(resp).await.map(|_| ())
    }

    /// POST /restart (reserved for future use)
    pub async fn restart(&self) -> Result<(), Error> {
        let url = self.restart_url();
        let resp = self
            .client
            .post(url)
            .header("content-type", "application/json")
            .body("{}")
            .send()
            .await
            .map_err(Error::Connection)?;
        check_response(resp).await.map(|_| ())
    }

    /// GET /status
    pub async fn status(&self) -> Result<StatusResponse, Error> {
        let url = self.status_url();
        let resp = self.client.get(url).send().await.map_err(Error::Connection)?;
        let resp = check_response(resp).await?;
        resp.json::<StatusResponse>().await.map_err(Error::DeserializeFailed)
    }
}

async fn check_response(resp: reqwest::Response) -> Result<reqwest::Response, Error> {
    let status = resp.status();
    if status.is_success() {
        debug!("[MatchComposerClient] Response OK ({status})");
        Ok(resp)
    } else {
        let body = resp.text().await.unwrap_or_default();
        Err(Error::RequestFailed {
            status: status.as_u16(),
            body,
        })
    }
}
