use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};
use crate::metadata::MetaData;
use super::{AppState, Error};

#[derive(Deserialize)]
pub struct StartRequest {
    #[serde(flatten)]
    pub config: Option<MetaData>,
}

#[derive(Serialize)]
pub struct StartResponse {

}

async fn post(
    State(state): State<AppState>,
    req: Option<Json<StartRequest>>,
) -> Result<Json<StartResponse>, Error> {
    let config = req.and_then(|Json(r)| r.config);
    state.start(config).await?;
    Ok(Json(StartResponse {

    }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
