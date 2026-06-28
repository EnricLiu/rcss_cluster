use std::borrow::Cow;
use std::collections::BTreeMap;
use std::net::IpAddr;
use std::ops::Deref;

use k8s_openapi::api::core::v1::PodTemplateSpec;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, Time};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServer {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: GameServerSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<GameServerStatus>,
}

impl GameServer {
    pub fn name(&self) -> &str {
        self.metadata.name.as_deref().unwrap_or("Anonymous GameServer")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<GameServerPortSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthSpec>,
    #[serde(rename = "sdkServer", skip_serializing_if = "Option::is_none")]
    pub sdk_server: Option<SdkServerSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<BTreeMap<String, CounterStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<BTreeMap<String, ListStatus>>,
    pub template: PodTemplateSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerPortSpec {
    pub name: String,
    #[serde(rename = "portPolicy")]
    pub port_policy: String,
    #[serde(rename = "containerPort")]
    pub container_port: i32,
    pub protocol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(rename = "initialDelaySeconds", skip_serializing_if = "Option::is_none")]
    pub initial_delay_seconds: Option<i32>,
    #[serde(rename = "periodSeconds", skip_serializing_if = "Option::is_none")]
    pub period_seconds: Option<i32>,
    #[serde(rename = "failureThreshold", skip_serializing_if = "Option::is_none")]
    pub failure_threshold: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SdkServerSpec {
    #[serde(rename = "logLevel", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(rename = "grpcPort", skip_serializing_if = "Option::is_none")]
    pub grpc_port: Option<i32>,
    #[serde(rename = "httpPort", skip_serializing_if = "Option::is_none")]
    pub http_port: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterStatus {
    pub count: i64,
    pub capacity: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i64>,
    #[serde(default)]
    pub values: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerStatus {
    pub state: String,
    #[serde(flatten)]
    pub connection: GameServerConnectionInfo,
    #[serde(rename = "reservedUntil", skip_serializing_if = "Option::is_none")]
    pub reserved_until: Option<Time>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eviction: Option<Eviction>,
}

impl Deref for GameServerStatus {
    type Target = GameServerConnectionInfo;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerConnectionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_as_default")]
    pub addresses: Vec<GameServerStatusAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<GameServerStatusPort>>,
    #[serde(rename = "nodeName", skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counters: Option<BTreeMap<String, CounterStatus>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lists: Option<BTreeMap<String, ListStatus>>,
}

impl GameServerStatus {
    pub fn is_ready(&self) -> bool {
        self.state == "Ready"
    }

    pub fn get_pod_ip(&self) -> Option<IpAddr> {
        self.connection.get_pod_ip()
    }
}

impl GameServerConnectionInfo {
    pub fn get_pod_ip(&self) -> Option<IpAddr> {
        for addr in &self.addresses {
            if let Some(pod_ip) = addr.as_pod_ip() {
                return Some(*pod_ip);
            }
        }
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "address")]
pub enum GameServerStatusAddress {
    ExternalDNS(String),
    InternalIP(IpAddr),
    ExternalIP(IpAddr),
    InternalDNS(String),
    Hostname(String),
    PodIP(IpAddr),
}

impl GameServerStatusAddress {
    pub fn as_pod_ip(&self) -> Option<&IpAddr> {
        match self {
            GameServerStatusAddress::PodIP(ip) => Some(ip),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerStatusPort {
    pub name: String,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Eviction {
    pub safe: String,
}

impl kube::Resource for GameServer {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn kind(_: &()) -> Cow<'_, str> {
        "GameServer".into()
    }

    fn group(_: &()) -> Cow<'_, str> {
        "agones.dev".into()
    }

    fn version(_: &()) -> Cow<'_, str> {
        "v1".into()
    }

    fn plural(_: &()) -> Cow<'_, str> {
        "gameservers".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

fn deserialize_null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_null_addresses_as_empty() {
        let status: GameServerStatus = serde_json::from_value(serde_json::json!({
            "state": "Starting",
            "address": "",
            "addresses": null,
            "ports": null
        }))
        .expect("GameServer status with null addresses should deserialize");

        assert!(status.addresses.is_empty());
        assert_eq!(status.state, "Starting");
    }
}
