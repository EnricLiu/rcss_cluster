mod create;

use std::collections::HashMap;

use uuid::Uuid;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use axum::extract::{Query, State};
use axum::{Json, Router};
use super::{AppState, Response};
use crate::model::team;
use crate::service::cluster;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub room_id: Uuid,
    pub team_name: String,
}

pub async fn get(
    State(s): State<AppState>,
    Query(req): Query<DeleteRequest>,
) -> Response {
    todo!()
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRequest {
    pub room_id: Uuid,
    pub team_name: String,
}

pub async fn delete(
    State(s): State<AppState>,
    Query(req): Query<DeleteRequest>,
) -> Response {
    // s.cluster.drop_team(req.room_id, req.team_name).await;
    todo!()
}


pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::delete(delete))
        .route("/", axum::routing::get(get))
        .merge(create::route("/create"))
        ;
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}