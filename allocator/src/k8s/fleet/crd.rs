use std::borrow::Cow;
use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::PodTemplateSpec;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fleet {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: FleetSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replicas: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<FleetStrategy>,
    #[serde(rename = "allocationOverflow", skip_serializing_if = "Option::is_none")]
    pub allocation_overflow: Option<AllocationOverflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priorities: Option<Vec<Priority>>,
    pub template: GameServerTemplateSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStrategy {
    #[serde(rename = "type")]
    pub strategy_type: String,
    #[serde(rename = "rollingUpdate", skip_serializing_if = "Option::is_none")]
    pub rolling_update: Option<RollingUpdateStrategy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollingUpdateStrategy {
    #[serde(rename = "maxSurge", skip_serializing_if = "Option::is_none")]
    pub max_surge: Option<String>,
    #[serde(rename = "maxUnavailable", skip_serializing_if = "Option::is_none")]
    pub max_unavailable: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllocationOverflow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Priority {
    #[serde(rename = "type")]
    pub priority_type: String,
    pub key: String,
    pub order: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerTemplateSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ObjectMeta>,
    pub spec: GameServerSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerSpec {
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
    #[serde(default)]
    pub values: Vec<String>,
}

impl kube::Resource for Fleet {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn kind(_: &()) -> Cow<'_, str> {
        "Fleet".into()
    }

    fn group(_: &()) -> Cow<'_, str> {
        "agones.dev".into()
    }

    fn version(_: &()) -> Cow<'_, str> {
        "v1".into()
    }

    fn plural(_: &()) -> Cow<'_, str> {
        "fleets".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}
