mod create;

use std::collections::HashMap;

use uuid::Uuid;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use axum::extract::{Query, State};
use axum::{Json, Router};

use super::{AppState, Response};
use crate::model::room;
use crate::service::cluster;

#[derive(Deserialize, Debug)]
pub struct GetRequest {
    pub id: Vec<Uuid>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum RoomResponse {
    Ok {
        #[serde(flatten)]
        room: room::Info,
    },
    Err {
        error: String,
    },
}

impl From<room::Info> for RoomResponse {
    fn from(r: room::Info) -> Self {
        RoomResponse::Ok { room: r }
    }
}
impl From<cluster::Result<room::Info>> for RoomResponse {
    fn from(r: cluster::Result<room::Info>) -> Self {
        match r {
            Ok(room) => RoomResponse::Ok { room },
            Err(cluster::Error::RoomNotFound { .. }) => {
                RoomResponse::Err {
                    error: "Room not found".to_string(),
                }
            },
            Err(_) => {
                RoomResponse::Err {
                    error: "Internal Server Error".to_string(),
                }
            },
        }
    }
}

#[derive(Serialize, Debug)]
pub struct GetResponse(HashMap<Uuid, RoomResponse>);

pub async fn get(
    State(s): State<AppState>,
    req: Option<Json<GetRequest>>
) -> Response {
    trace!("[http::room::mod] GET /room: {:?}", req);
    let resp = {
        if let Some(Json(GetRequest { id: room_ids })) = req {
            let mut tasks = Vec::with_capacity(room_ids.len());
            for id in room_ids {
                let cluster = s.cluster.clone();
                tasks.push(async move {
                    let res = cluster.room_info(id).await;
                    (id, res)
                })
            }

            let rooms: HashMap<_, _> = futures::future::join_all(tasks).await
                .into_iter().map(|r| (r.0, r.1.into())).collect();

            GetResponse(rooms)

        } else {
            let rooms = match s.cluster.all_rooms_info().await {
                Ok(rooms) => rooms,
                Err(e) => return e.into(),
            };

            let rooms = rooms
                .into_iter().map(|r| (r.room_id, r.into())).collect();

            GetResponse(rooms)

        }
    };

    Response::success(Some(resp))
}

#[derive(Deserialize, Debug)]
pub struct DeleteRequest {
    pub id: Uuid,
}

pub async fn delete(
    State(s): State<AppState>,
    Query(req): Query<DeleteRequest>
) -> Response {
    let ret = s.cluster.drop_room(req.id);
    debug!("/room/delete: dropping Room[{}], {ret:?}", req.id);
    match ret {
        Ok(_)  => Response::success::<()>(None),
        Err(_) => Response::code_u16(404),
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/", axum::routing::delete(delete))
        .route("/", axum::routing::get(get))
        .merge(create::route("/create"))
        // .merge(detail::route("/detail"))
    ;
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}