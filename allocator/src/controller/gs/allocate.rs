use std::net::IpAddr;
use std::collections::HashMap;

use log::{error, info, warn};
use axum::{extract::State, routing, Json, Router};
use serde::{Deserialize, Serialize};
use common::errors::BuilderError;

use crate::schema::v1;
use crate::k8s::{AllocationError, Error, GsAllocation};

use super::{AppState, Response};


#[derive(Deserialize, Debug)]
pub struct PostRequest {
    pub schema_v1: Option<v1::ConfigV1>,
}

#[derive(Serialize, Debug)]
pub struct PostResponse {
    pub name: String,
    pub host: IpAddr,
    pub ports: HashMap<String, u16>,
}

#[derive(Debug)]
struct ParsedPostRequest {
    pub meta: crate::MetaData,
}

impl TryFrom<PostRequest> for ParsedPostRequest {
    type Error = BuilderError;

    fn try_from(req: PostRequest) -> Result<Self, Self::Error> {
        let meta = match (req.schema_v1) {
            None => return Err(BuilderError::MissingField { field: "schema_v*" }),
            Some(schema) => schema.try_into()?,
        };

        let ret = Self {
            meta
        };

        Ok(ret)
    }
}

pub async fn post(
    State(state): State<AppState>,
    Json(req): Json<PostRequest>,
) -> Response {
    let req: ParsedPostRequest = match req.try_into() {
        Ok(req) => req,
        Err(err) => return Response::error("Invalid request", &err.to_string()),
    };

    let success = |res: GsAllocation| {
        Response::success(
            PostResponse {
                name: res.name,
                host: res.host,
                ports: res.ports,
            }
        )
    };

    let error = |e: &Error| {
        Response::error(e.desc(), &e.to_string())
    };

    // retry within the k8s allocation
    let alloc_res = state.k8s.gs_allocate(
        state.config.scheduling.clone(),
        req.meta.clone(),
    ).await;

    match alloc_res {
        Ok(res) => {
            info!("Allocation successful: {:?}", res);
            return success(res)
        },
        Err(AllocationError::UnAllocated) => {
            info!("Allocation failed due to no available GameServer, request meta: {:?}", req.meta);
        },
        Err(e) => {
            warn!("Allocation failed: {:?}", e);
            return error(&(e.into()))
        }
    };

    // UnAllocated here, check the cause
    let has_fleet = match state.k8s.fleet_exists_by_labels(&req.meta.labels).await {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to check fleet existence: {:?}", e);
            return error(&(e.into()))
        }
    };

    if has_fleet {
        info!("Fleet exists but no available GameServer, request meta: {:?}", req.meta);
        return error(&AllocationError::Busy.into())
    }

    // no supporting fleet here
    if let Err(e) = state.k8s.create_fleet_by_meta(
        format!("fleet-{}", uuid::Uuid::new_v4()),
        req.meta.clone(),
    ).await {
        error!("Failed to create fleet: {:?}", e);
        return error(&(e.into()))
    };

    // reallocate the GS
    let alloc_res = state.k8s.gs_allocate(
        state.config.scheduling.clone(),
        req.meta.clone(),
    ).await;

    match alloc_res {
        Ok(res) => {
            info!("Allocation successful after fleet creation: {:?}", res);
            success(res)
        },
        Err(e) => {
            warn!("Allocation failed even fleet had been created: {:?}", e);
            error(&(e.into()))
        }
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", routing::post(post));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
