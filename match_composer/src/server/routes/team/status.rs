use axum::{routing, Router};
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use common::axum::response::Response;
use common::types::Side;

use crate::info::TeamInfo;
use super::{AppState, Error};

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
) -> Response {
    let side = req.side;
    let info = state.team_info(side).await;
    
    match info {
        Some(info) => Response::success(Some(GetResponse { info })),
        None => Error::BadRequest("not running".to_string()).into(),
        _ => Error::Internal("wtf".to_string()).into(),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    Router::new().route(path, routing::get(get))
}
