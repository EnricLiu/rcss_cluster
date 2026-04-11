use axum::{routing, Json, Router};
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

use common::types::Side;

use crate::info::TeamInfo;
use super::{AppState, Result, Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetRequest {
    side: Side,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    #[serde(flatten)]
    info: TeamInfo
}

async fn get(
    State(state): State<AppState>,
    Query(req): Query<GetRequest>,
) -> Result<Json<GetResponse>> {
    let side = req.side;
    let info = state.team_info(side).await;
    
    match info {
        Some(info) => Ok(GetResponse { info }.into()),
        None => Err(Error::BadRequest("not running".to_string())),
        _ => Err(Error::Internal("wtf".to_string())),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
