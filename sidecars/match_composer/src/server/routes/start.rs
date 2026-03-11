use axum::extract::State;
use axum::{Json, Router, routing};
use serde::Deserialize;

use crate::schema::v1::ConfigV1;
use super::super::{AppState, Error};
use super::super::response::{StartResponse};

#[derive(Deserialize)]
pub struct StartRequest {
    #[serde(flatten)]
    pub config: Option<ConfigV1>,
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
