use std::borrow::Cow;
use std::collections::BTreeMap;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};

use super::GameServerSpec;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fleet {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: FleetSpec,
    pub status: Option<FleetStatus>,
}

impl Fleet {
    pub fn name(&self) -> &str {
        self.metadata.name.as_deref().unwrap_or("Anonymous Fleet")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStatus {
    pub replicas: usize,
    #[serde(rename = "allocatedReplicas")]
    pub allocated_replicas: usize,
    #[serde(rename = "readyReplicas")]
    pub ready_replicas: usize,
    #[serde(rename = "reservedReplicas")]
    pub reserved_replicas: usize,
}

impl FleetStatus {
    pub fn has_ready(&self) -> bool {
        self.ready_replicas > 0
    }
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
