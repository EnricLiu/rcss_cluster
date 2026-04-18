use std::fmt::Debug;

use kube::Api;
use kube::api::PostParams;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use log::{debug, info};
use common::errors::BuilderError;
use crate::MetaData;
use crate::args::Scheduling;
use crate::k8s::crd::AllocationState;
use crate::k8s::crd::AllocationState::{Allocated, Contention, Unallocated};
use super::crd::{
    AllocationMetadata, GameServerAllocation,
    GameServerAllocationSpec, GameServerSelector,
};
use super::builder::{GsAllocation, GsAllocationBuilder};

use super::{Error, Result, K8sClient};

impl K8sClient {
    pub async fn gs_allocate(
        &self,
        scheduling: Scheduling,
        metadata: impl TryInto<MetaData, Error: Debug>,
    ) -> AllocationResult<GsAllocation> {
        let metadata = metadata.try_into()
            .map_err(|e| AllocationError::InvalidMetaData(format!("{e:?}")))?;

        // Build allocation metadata with annotations
        let allocation_metadata = AllocationMetadata {
            annotations: Some(metadata.annotations.into_map()),
        };

        // Build selector to match fleet
        let match_labels = {
            let mut labels = metadata.labels.try_into_map()
                .map_err(|e| AllocationError::InvalidMetaData(format!("{e:?}")))?;
            // labels.insert("agones.dev/fleet".to_string(), fleet_name.to_string());
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
                scheduling: Some(scheduling.as_str().to_string()),
                metadata: Some(allocation_metadata),
            },
            status: None,
        }; // TODO builder

        let api: Api<GameServerAllocation> = Api::namespaced(self.client.clone(), &self.agones_ns);
        for _ in 1..=self.n_retry_human() {
            match make_allocation(&api, &allocation).await {
                Ok(res) => return Ok(res),
                Err(AllocationError::Busy) => {
                    info!("Allocation request failed due to contention, retrying...");
                    continue;
                }
                Err(e) => return Err(e),
            }
        };

        Err(AllocationError::Busy)
    }
}

async fn make_allocation(
    api: &Api<GameServerAllocation>,
    allocation: &GameServerAllocation
) -> AllocationResult<GsAllocation> {

    let result = api.create(&PostParams::default(), allocation).await?;

    let status = result.status
        .ok_or(AllocationError::BadResponse(BuilderError::MissingField { field: "status" }))?;

    use AllocationState::*;
    match &status.state {
        Allocated => {
            debug!("Allocation request `allocated`, GS allocated successfully");
        }
        Unallocated => {
            debug!("Allocation request `unallocated`, no GS available for the allocation");
            return Err(AllocationError::UnAllocated)
        },
        Contention => {
            debug!("Allocation request `contention`, Agones API busy");
            return Err(AllocationError::Busy)
        }
    }

    let res = {
        let mut builder = GsAllocationBuilder::new();
        builder
            .parse_host(status.address.as_ref()).map_err(AllocationError::BadResponse)?
            .parse_ports(status.ports.unwrap_or_default())
            .set_name(status.game_server_name.clone());
        builder.build_into().map_err(AllocationError::BadResponse)?
    };

    Ok(res)
}

#[derive(thiserror::Error, Debug)]
pub enum AllocationError {
    #[error("Agones API busy, pls retry later")]
    Busy,
    #[error("No GS available for the allocation")]
    UnAllocated,
    #[error("Failed to parse the GSA response, {0}")]
    BadResponse(#[from] BuilderError),
    #[error("Failed to make a GSA request, {0}")]
    ApiError(#[from] kube::Error),
    #[error("Invalid metadata to build the GSA, {0}")]
    InvalidMetaData(String),
}

impl AllocationError {
    pub fn desc(&self) -> &'static str {
        match self {
            AllocationError::UnAllocated => "UnAllocated",
            AllocationError::Busy => "BusyContention",
            AllocationError::ApiError(_) => "ApiError",
            AllocationError::BadResponse(_) => "BadResponse",
            AllocationError::InvalidMetaData(_) => "InvalidMetaData",
        }
    }
}

pub type AllocationResult<T> = std::result::Result<T, AllocationError>;
