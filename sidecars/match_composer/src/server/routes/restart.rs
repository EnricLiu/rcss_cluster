use axum::extract::State;
use axum::{Json, Router, routing};
use serde::Deserialize;

use crate::schema::v1::ConfigV1;
use super::super::{AppState, Error};
use super::super::response::StartResponse;

#[derive(Deserialize)]
pub struct RestartRequest {
    #[serde(flatten)]
    pub config: Option<ConfigV1>,
}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<RestartRequest>,
) -> Result<Json<StartResponse>, Error> {
    state.restart(req.config).await?;
    Ok(Json(StartResponse {
        
    }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
