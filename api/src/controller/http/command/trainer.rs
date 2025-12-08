use serde::{Deserialize, Serialize};
use axum::{routing, Json, Router};
use axum::extract::State;

use common::command::{Command, CommandResult};
use common::command::trainer::*;

use super::{AppState, Response};
use super::CommandResponse;

impl AppState {
    pub async fn send_trainer_command<C: Command<Kind = TrainerCommand>>(
        &self,
        cmd: C,
    ) -> CommandResult<C> {
        self.service.send_trainer_command(cmd).await
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostRequest<C: Command<Kind=TrainerCommand>>(C);
#[derive(Debug)]
pub struct PostResponse<C: Command<Kind=TrainerCommand>>(CommandResponse<C>);

impl<C: Command<Kind=TrainerCommand>> Serialize for PostResponse<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

pub async fn post<C: Command<Kind=TrainerCommand>>(
    State(s): State<AppState>,
    Json(req): Json<PostRequest<C>>,
) -> Json<Response> {
    let result: CommandResponse<C> = s.send_trainer_command(req.0).await.into();
    let resp = PostResponse(result);
    Json(Response::success(Some(resp)))
}
pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/change_mode", routing::post(post::<ChangeMode>))
        .route("/check_ball", routing::post(post::<CheckBall>))
        .route("/ear", routing::post(post::<Ear>))
        .route("/eye", routing::post(post::<Eye>))
        .route("/init", routing::post(post::<Init>))
        .route("/look", routing::post(post::<Look>))
        .route("/move", routing::post(post::<Move>))
        .route("/recover", routing::post(post::<Recover>))
        .route("/start", routing::post(post::<Start>))
        .route("/team_names", routing::post(post::<TeamNames>));
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
