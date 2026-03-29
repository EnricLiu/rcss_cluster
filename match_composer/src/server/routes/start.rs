use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};
use crate::agones::AgonesMetaData;
use super::{AppState, Error};

#[derive(Deserialize)]
pub struct StartRequest {
    #[serde(flatten)]
    pub config: Option<AgonesMetaData>,
}

#[derive(Serialize)]
pub struct StartResponse {

}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<StartRequest>,
) -> Result<Json<StartResponse>, Error> {
    state.start(req.config).await?;
    Ok(Json(StartResponse {

    }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
