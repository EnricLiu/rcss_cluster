mod create;
mod room;

use axum::extract::{Query, State};
use axum::{routing, Json, Router};
use serde::{Deserialize, Serialize};

use crate::room::RoomInfo;
use super::{AppState, Response, Error};

#[derive(Debug, Clone, Serialize)]
pub struct RoomResponse {
    pub name: String,
    pub player_udp: String,
    pub trainer_udp: String,
    pub coach_udp: String,
    pub ws_url: String,
}

impl From<RoomInfo> for RoomResponse {
    fn from(info: RoomInfo) -> Self {
        Self {
            name: info.config.name,
            player_udp: info.config.player_udp.to_string(),
            trainer_udp: info.config.trainer_udp.to_string(),
            coach_udp: info.config.coach_udp.to_string(),
            ws_url: info.config.ws.base_url.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GetRequest {
    pub offset: usize,
    pub limit: usize,
}
impl Default for GetRequest {
    fn default() -> Self {
        Self { offset: 0, limit: 20, }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GetResponse {
    pub rooms: Vec<RoomResponse>,
    pub total: usize,
}

pub async fn get(
    State(state): State<AppState>,
    Query(req): Query<GetRequest>,
) -> Response {
    let rooms: Vec<RoomResponse> = state.server
        .all_room_infos()
        .into_iter()
        .skip(req.offset)
        .take(req.limit)
        .map(RoomResponse::from)
        .collect();

    let total = state.server.room_count();
    
    Response::success(Some(GetResponse { rooms, total }))
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .merge(create::route("/create"))
        .merge(room::route("/{name}"))
        .route("/", routing::get(get));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}