pub mod stop;
pub mod start;
pub mod status;
pub mod restart;
pub mod team;

use axum::Router;
use super::{AppState, Result, Error};

pub fn route(path: &str, state: AppState) -> Router {
    let inner = Router::new()
        .merge(start::route("/start"))
        .merge(stop::route("/stop"))
        .merge(restart::route("/restart"))
        .merge(status::route("/status"))
        .merge(team::route("/team"))
        .with_state(state);
    
    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
