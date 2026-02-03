use axum::extract::State;
use axum::{Json, Router, routing};
use common::command::Command;
use common::command::trainer::*;
use serde::{Deserialize, Serialize};

use super::CommandResponse;
use super::{AppState, Error, Response};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PostRequest<C: Command<Kind = TrainerCommand>>(C);
#[derive(Debug)]
pub struct PostResponse<C: Command<Kind = TrainerCommand>>(CommandResponse<C>);

impl<C: Command<Kind = TrainerCommand>> Serialize for PostResponse<C> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

pub async fn post<C: Command<Kind = TrainerCommand>>(
    State(s): State<AppState>,
    Json(req): Json<PostRequest<C>>,
) -> Json<Response> {
    let result = s.service.send_trainer_command(req.0).await;
    if let Err(e) = result {
        return Json(Error::from(e).into());
    }

    let result: CommandResponse<C> = result.expect("WTF").into();
    let resp = PostResponse(result);
    println!("{:?}", serde_json::to_string(&resp));
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

#[cfg(test)]
mod test {
    use super::*;
    use common::command::trainer::Start;

    #[test]
    fn test_se_des() {
        let req = PostRequest(Start);
        let json = serde_json::to_string(&req).unwrap();
        let req: PostRequest<Start> = serde_json::from_str("[null]").unwrap();
        println!("{req:?}")
    }
}
