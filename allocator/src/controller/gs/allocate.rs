use std::collections::HashMap;
use std::net::IpAddr;

use axum::{extract::State, routing, Json, Router};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::schema::v1;
use super::{AppState, Response};


#[derive(Deserialize, Debug)]
pub struct PostRequest {
    pub fleet: Option<String>,
    #[serde(flatten)]
    pub schema: v1::ConfigV1,
}

#[derive(Serialize, Debug)]
pub struct PostResponse {
    pub name: String,
    pub host: IpAddr,
    pub ports: HashMap<String, u16>,
}

pub async fn post(
    State(state): State<AppState>,
    Json(PostRequest{ fleet, schema }): Json<PostRequest>,
) -> Response {
    // TODO
    let fleet_name = fleet.unwrap_or_else(|| format!("fleet-{}", Uuid::new_v4()));
    let res = state.k8s.allocate(
        &fleet_name,
        state.config.scheduling.as_str(),
        schema.clone(),
    ).await.expect("TODO");

    Response::success(Some(
        PostResponse {
            name: res.name,
            host: res.host,
            ports: res.ports,
        }
    ))
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
