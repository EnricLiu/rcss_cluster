use std::fmt::Debug;

use kube::Api;
use kube::api::PostParams;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

use common::errors::BuilderError;

use crate::MetaData;
use super::crd::{
    AllocationMetadata, GameServerAllocation,
    GameServerAllocationSpec, GameServerSelector,
};
use super::builder::{GsAllocation, GsAllocationBuilder};

use super::{Error, Result, K8sClient};

impl K8sClient {
    pub async fn allocate(
        &self,
        fleet_name: &str,
        scheduling: &str,
        metadata: impl TryInto<MetaData, Error: Debug>,
    ) -> Result<GsAllocation> {
        let api: Api<GameServerAllocation> = Api::namespaced(self.client.clone(), &self.agones_ns);

        let metadata = metadata.try_into()
            .map_err(|e| Error::InvalidMetaData(format!("{e:?}")))?;

        // Build allocation metadata with annotations
        let allocation_metadata = AllocationMetadata {
            annotations: Some(metadata.annotations.into_map()),
        };

        // Build selector to match fleet
        let match_labels = {
            let mut labels = metadata.labels.into_map();
            labels.insert("agones.dev/fleet".to_string(), fleet_name.to_string());
            labels
        };

        let selector = GameServerSelector {
            match_labels: Some(match_labels),
            match_expressions: None,
        };

        // Create allocation request
        let allocation = GameServerAllocation {
            api_version: "allocation.agones.dev/v1".to_string(),
            kind: "GameServerAllocation".to_string(),
            metadata: ObjectMeta {
                namespace: Some(self.agones_ns.to_string()),
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
        let result = api.create(&PostParams::default(), &allocation).await
            .map_err(Error::NoSuchGs)?;

        // Parse response
        let status = result.status
            .ok_or(Error::GsaBadResponse(BuilderError::MissingField { field: "status" }))?;

        // Check allocation state
        if status.state != "Allocated" {
            return Err(Error::GsaExhausted(
                "当前训练资源已满，请稍后重试".to_string(),
            ));
        }

        let res = {
            let mut builder = GsAllocationBuilder::new();
            builder
                .parse_host(status.address.as_ref()).map_err(Error::GsaBadResponse)?
                .parse_ports(status.ports.unwrap_or_default())
                .set_name(status.game_server_name.clone());
            builder.build_into().map_err(Error::GsaBadResponse)?
        };

        Ok(res)
    }
}

