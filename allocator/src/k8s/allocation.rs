use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, PostParams},
    Client,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::controller::{error::Error, response::AllocateResponse};

// GameServerAllocation CRD structures
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
pub struct GameServerAllocationStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<GameServerPort>>,
    #[serde(rename = "gameServerName", skip_serializing_if = "Option::is_none")]
    pub game_server_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameServerPort {
    pub name: String,
    pub port: u16,
}

impl kube::Resource for GameServerAllocation {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn kind(_: &()) -> std::borrow::Cow<'_, str> {
        "GameServerAllocation".into()
    }

    fn group(_: &()) -> std::borrow::Cow<'_, str> {
        "allocation.agones.dev".into()
    }

    fn version(_: &()) -> std::borrow::Cow<'_, str> {
        "v1".into()
    }

    fn plural(_: &()) -> std::borrow::Cow<'_, str> {
        "gameserverallocations".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

pub async fn create_allocation(
    client: &Client,
    namespace: &str,
    fleet_name: &str,
    scheduling: &str,
    annotations: HashMap<String, String>,
) -> Result<AllocateResponse, Error> {
    let api: Api<GameServerAllocation> = Api::namespaced(client.clone(), namespace);

    // Build allocation metadata with annotations
    let allocation_metadata = AllocationMetadata {
        annotations: Some(annotations),
    };

    // Build selector to match fleet
    let match_labels = HashMap::from([("agones.dev/fleet".to_string(), fleet_name.to_string())]);

    let selector = GameServerSelector {
        match_labels: Some(match_labels),
        match_expressions: None,
    };

    // Create allocation request
    let allocation = GameServerAllocation {
        api_version: "allocation.agones.dev/v1".to_string(),
        kind: "GameServerAllocation".to_string(),
        metadata: ObjectMeta {
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: GameServerAllocationSpec {
            selectors: vec![selector],
            scheduling: Some(scheduling.to_string()),
            metadata: Some(allocation_metadata),
        },
        status: None,
    };

    // Submit allocation
    let result = api.create(&PostParams::default(), &allocation).await?;

    // Parse response
    let status = result.status.ok_or_else(|| {
        Error::Internal("GameServerAllocation response missing status".to_string())
    })?;

    // Check allocation state
    if status.state != "Allocated" {
        return Err(Error::ResourceExhausted(
            "当前训练资源已满，请稍后重试".to_string(),
        ));
    }

    // Extract address and port
    let address = status
        .address
        .ok_or_else(|| Error::Internal("Missing address in allocation response".to_string()))?;

    let ports = status
        .ports
        .ok_or_else(|| Error::Internal("Missing ports in allocation response".to_string()))?;

    let port = ports
        .first()
        .ok_or_else(|| Error::Internal("No ports in allocation response".to_string()))?
        .port;

    let game_server_name = status.game_server_name.ok_or_else(|| {
        Error::Internal("Missing gameServerName in allocation response".to_string())
    })?;

    Ok(AllocateResponse {
        ip: address,
        port,
        game_server_name,
    })
}
