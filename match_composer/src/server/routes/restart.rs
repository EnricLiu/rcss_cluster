use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};
use crate::agones::AgonesMetaData;
use super::{AppState, Error};

#[derive(Deserialize)]
pub struct RestartRequest {
    #[serde(flatten)]
    pub config: Option<AgonesMetaData>,
}

#[derive(Serialize)]
pub struct RestartResponse {
    
}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<RestartRequest>,
) -> Result<Json<RestartResponse>, Error> {
    state.restart(req.config).await?;
    Ok(Json(RestartResponse {
        
    }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
