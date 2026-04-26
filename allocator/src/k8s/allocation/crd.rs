use std::net::IpAddr;
use std::borrow::Cow;
use std::collections::HashMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerAllocation {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: GameServerAllocationSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<GameServerAllocationStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerAllocationSpec {
    pub selectors: Vec<GameServerSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AllocationMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerSelector {
    #[serde(rename = "matchLabels", skip_serializing_if = "Option::is_none")]
    pub match_labels: Option<HashMap<String, String>>,
    #[serde(rename = "matchExpressions", skip_serializing_if = "Option::is_none")]
    pub match_expressions: Option<Vec<LabelSelectorRequirement>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    pub key: String,
    pub operator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllocationMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AllocationState {
    #[serde(rename = "UnAllocated")]
    Unallocated,
    #[serde(rename = "Allocated")]
    Allocated,
    #[serde(rename = "Contention")]
    Contention,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerAllocationStatus {
    pub state: AllocationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default)]
    pub addresses: Vec<GameServerAllocationStatusAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<GameServerPort>>,
    #[serde(rename = "gameServerName", skip_serializing_if = "Option::is_none")]
    pub game_server_name: Option<String>,
}

impl GameServerAllocationStatus {
    pub fn get_pod_ip(&self) -> Option<IpAddr> {
        for addr in &self.addresses {
            let maybe_pod_ip = addr.as_pod_ip();
            if let Some(pod_ip) = maybe_pod_ip {
                return Some(*pod_ip);
            }
        }
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "address")]
pub enum GameServerAllocationStatusAddress {
    InternalIP(IpAddr),
    ExternalIP(IpAddr),
    Hostname(String),
    PodIP(IpAddr),
}

impl GameServerAllocationStatusAddress {
    pub fn as_pod_ip(&self) -> Option<&IpAddr> {
        match self {
            GameServerAllocationStatusAddress::PodIP(ip) => Some(ip),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerPort {
    pub name: String,
    pub port: u16,
}

impl kube::Resource for GameServerAllocation {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn kind(_: &()) -> Cow<'_, str> {
        "GameServerAllocation".into()
    }

    fn group(_: &()) -> Cow<'_, str> {
        "allocation.agones.dev".into()
    }

    fn version(_: &()) -> Cow<'_, str> {
        "v1".into()
    }

    fn plural(_: &()) -> Cow<'_, str> {
        "gameserverallocations".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}