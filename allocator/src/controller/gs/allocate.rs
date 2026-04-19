use std::net::IpAddr;
use std::collections::HashMap;
use log::{debug, error, info, warn};
use axum::{extract::State, routing, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use common::errors::BuilderError;

use crate::schema::{v1, Schema};
use crate::k8s::{AllocationError, Error, GsAllocation};

use super::{AppState, Response};


#[derive(Deserialize, Debug)]
pub struct PostRequest {
    pub version: u8,
    pub conf: Value,
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
    type Error = Error;

    fn try_from(req: PostRequest) -> Result<Self, Self::Error> {
        fn err<E: std::fmt::Debug>(e: &E) -> Error {
            Error::InvalidMetaData(format!("Failed to parse request conf into metadata, error: {:?}", e))
        }

        let meta = match (req.version) {
            1 => {
                let conf: v1::ConfigV1 = serde_json::from_value(req.conf)
                    .map_err(|e| err(&BuilderError::InvalidValue {
                        field: "conf",
                        value: e.to_string(),
                        expected: "a valid ConfigV1".to_string(),
                    }))?;

                conf.verify().map_err(|e| err(&BuilderError::InvalidValue {
                    field: "conf",
                    value: e.to_string(),
                    expected: "a valid ConfigV1".to_string(),
                }))?;

                conf.try_into().map_err(|e| err(&e))?
            },

            _ => return Err(err(&BuilderError::InvalidValue {
                field: "version",
                value: req.version.to_string(),
                expected: "supported version, currently only 1".to_string(),
            })),
        };

        Ok(
            Self {
                meta
            }
        )
    }
}

/// could be in really long time
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

    match state.k8s.get_or_create_fleet_by_meta(req.meta.clone(), None).await {
        Ok(fleet) => {
            debug!("Fleet[{}] is ready for allocation", fleet.name());
        },
        Err(e) => {
            warn!("Failed to get or create fleet, error: {:?}", e);
            return error(&e.into())
        }
    }

    let alloc_res = state.k8s.gs_allocate(
        state.config.scheduling.clone(),
        req.meta,
    ).await;

    match alloc_res {
        Ok(res) => {
            info!("Allocation successful: {res:?}");
            success(res)
        },
        Err(e) => {
            warn!("Allocation failed: {e:?}");
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
