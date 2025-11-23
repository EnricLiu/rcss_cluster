use uuid::Uuid;
use axum::extract::State;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use super::{AppState, Response};

use crate::service::{room, team};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostRequest {
    pub room_id: Uuid,
    #[serde(flatten)]
    pub config: team::Config,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostResponse {
    pub room_id: Uuid,
    pub name: String,
}

pub async fn post(
    State(s): State<AppState>,
    Json(req): Json<PostRequest>
) -> Response {
    let team_name = match s.cluster.create_team(req.room_id, req.config).await {
        Ok(team_name) => team_name,
        Err(e) => return e.into(),
    };

    let resp = PostResponse {
        room_id: req.room_id,
        name: team_name,
    };

    Response::success(Some(resp))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new()
        .route(path, axum::routing::post(post))
}