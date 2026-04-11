use axum::extract::State;
use axum::{Json, Router, routing};
use serde::{Deserialize, Serialize};
use crate::metadata::MetaData;
use super::{AppState, Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostRequest {
    #[serde(flatten)]
    pub config: Option<MetaData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostResponse {
    
}

async fn post(
    State(state): State<AppState>,
    Json(req): Json<PostRequest>,
) -> Result<Json<PostResponse>, Error> {
    state.restart(req.config).await?;
    Ok(Json(PostResponse {
        
    }))
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::post(post))
}
