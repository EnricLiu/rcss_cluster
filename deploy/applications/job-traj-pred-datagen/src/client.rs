use crate::error::{Error, Result};
use reqwest::Method;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

#[derive(Clone)]
pub struct AllocatorClient {
    http: reqwest::Client,
    allocate_url: String,
    heartbeat_url: String,
    drop_url: String,
}

impl AllocatorClient {
    pub fn new(http: reqwest::Client, allocator_url: &str) -> Self {
        let gs_base = normalize_gs_base(allocator_url);
        Self {
            http,
            allocate_url: format!("{gs_base}/allocate"),
            heartbeat_url: format!("{gs_base}/heartbeat"),
            drop_url: format!("{gs_base}/drop"),
        }
    }

    pub async fn allocate(&self, request: &Value) -> Result<Allocation> {
        send_api(
            self.http.post(&self.allocate_url).json(request),
            "allocator_allocate",
        )
        .await
    }

    pub async fn heartbeat(&self, name: &str) -> Result<()> {
        let _: Value = send_api(
            self.http
                .post(&self.heartbeat_url)
                .json(&NameRequest { name }),
            "allocator_heartbeat",
        )
        .await?;
        Ok(())
    }

    pub async fn drop_direct_gs(&self, name: &str) -> Result<()> {
        let _: Value = send_api(
            self.http
                .request(Method::DELETE, &self.drop_url)
                .json(&NameRequest { name }),
            "allocator_drop",
        )
        .await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct GameServerClient {
    http: reqwest::Client,
    base_urls: Vec<String>,
}

impl GameServerClient {
    pub fn from_allocation(
        http: reqwest::Client,
        allocation: &Allocation,
        prefer_pod_ip: bool,
        pod_http_port: u16,
    ) -> Result<Self> {
        let mut pod_base = Some(format!("http://{}:{pod_http_port}", allocation.pod));
        let host_base = allocation
            .ports
            .get("default")
            .copied()
            .map(|port| format!("http://{}:{port}", allocation.host));

        if host_base.is_none() {
            return Err(Error::MissingPort("default"));
        }

        let mut base_urls = Vec::with_capacity(2);
        if prefer_pod_ip {
            if let Some(base) = pod_base.take() {
                base_urls.push(base);
            }
            if let Some(base) = host_base {
                base_urls.push(base);
            }
        } else {
            if let Some(base) = host_base {
                base_urls.push(base);
            }
            if let Some(base) = pod_base.take() {
                base_urls.push(base);
            }
        }

        Ok(Self { http, base_urls })
    }

    pub fn primary_base_url(&self) -> Option<&str> {
        self.base_urls.first().map(String::as_str)
    }

    pub async fn status(&self) -> Result<ServerObservation> {
        self.try_each_base("server_status", |http, base| {
            let url = format!("{base}/metrics/status");
            async move { send_api(http.get(url), "server_status").await }
        })
        .await
    }

    pub async fn trainer_start(&self) -> Result<Value> {
        self.try_each_base("trainer_start", |http, base| {
            let url = format!("{base}/trainer/start");
            async move { send_api(http.post(url).json(&serde_json::json!({})), "trainer_start").await }
        })
        .await
    }

    pub async fn shutdown(&self) -> Result<Value> {
        self.try_each_base("server_shutdown", |http, base| {
            let url = format!("{base}/control/shutdown");
            async move {
                send_api(
                    http.post(url).json(&serde_json::json!({ "force": true })),
                    "server_shutdown",
                )
                .await
            }
        })
        .await
    }

    async fn try_each_base<T, F, Fut>(&self, op: &'static str, f: F) -> Result<T>
    where
        F: Fn(reqwest::Client, String) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut last_error = None;
        for base in &self.base_urls {
            match f(self.http.clone(), base.clone()).await {
                Ok(value) => return Ok(value),
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| Error::Api {
            op,
            message: "no GameServer HTTP base URL is configured".to_string(),
        }))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Allocation {
    pub name: String,
    pub pod: IpAddr,
    pub host: IpAddr,
    pub ports: HashMap<String, u16>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerObservation {
    pub service: ServiceObservation,
    #[serde(default)]
    pub conn_count: Option<usize>,
    #[serde(default)]
    pub agones: Option<AgonesObservation>,
}

impl ServerObservation {
    pub fn is_started(&self) -> bool {
        self.service.status.eq_ignore_ascii_case("simulating")
            || self.service.timestep.unwrap_or_default() > 0
    }

    pub fn is_finished(&self, time_up: u16) -> bool {
        self.service.status.eq_ignore_ascii_case("finished")
            || self.service.timestep.is_some_and(|t| t >= time_up)
    }

    pub fn match_composer_ready(&self) -> bool {
        self.agones
            .as_ref()
            .and_then(|a| a.mc_last_poll.as_ref())
            .is_some_and(|poll| poll.in_match)
            || self.conn_count.unwrap_or_default() > 0
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceObservation {
    pub status: String,
    #[serde(default)]
    pub timestep: Option<u16>,
    #[serde(default)]
    pub process_status: Option<String>,
    #[serde(default)]
    pub uptime_ms: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgonesObservation {
    #[serde(default)]
    pub mc_last_poll: Option<McLastPollObservation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McLastPollObservation {
    pub in_match: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Serialize)]
struct NameRequest<'a> {
    name: &'a str,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    success: bool,
    payload: Value,
}

async fn send_api<T>(request: reqwest::RequestBuilder, op: &'static str) -> Result<T>
where
    T: DeserializeOwned,
{
    let response = request
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|source| Error::http(op, source))?;

    let envelope = response
        .json::<ApiEnvelope>()
        .await
        .map_err(|source| Error::http(op, source))?;

    if !envelope.success {
        return Err(Error::Api {
            op,
            message: api_error_message(&envelope.payload),
        });
    }

    serde_json::from_value(envelope.payload).map_err(|source| Error::decode(op, source))
}

fn api_error_message(payload: &Value) -> String {
    payload
        .get("desc")
        .and_then(Value::as_str)
        .or_else(|| payload.get("error").and_then(Value::as_str))
        .map(ToString::to_string)
        .unwrap_or_else(|| payload.to_string())
}

fn normalize_gs_base(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    if trimmed.ends_with("/gs/allocate") {
        trimmed.trim_end_matches("/allocate").to_string()
    } else if trimmed.ends_with("/gs") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/gs")
    }
}

pub async fn sleep_interval(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_allocator_routes() {
        assert_eq!(
            normalize_gs_base("http://allocator/gs/allocate"),
            "http://allocator/gs"
        );
        assert_eq!(
            normalize_gs_base("http://allocator/gs/"),
            "http://allocator/gs"
        );
        assert_eq!(normalize_gs_base("http://allocator"), "http://allocator/gs");
    }

    #[test]
    fn observes_started_and_finished_status() {
        let obs = ServerObservation {
            service: ServiceObservation {
                status: "idle".to_string(),
                timestep: Some(10),
                process_status: None,
                uptime_ms: None,
            },
            conn_count: None,
            agones: None,
        };
        assert!(obs.is_started());
        assert!(!obs.is_finished(6000));

        let obs = ServerObservation {
            service: ServiceObservation {
                status: "finished".to_string(),
                timestep: Some(6000),
                process_status: None,
                uptime_ms: None,
            },
            conn_count: None,
            agones: None,
        };
        assert!(obs.is_finished(6000));
    }
}
