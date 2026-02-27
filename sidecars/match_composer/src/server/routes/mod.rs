mod restart;
mod start;
mod status;
mod stop;

use axum::Router;
use super::AppState;

pub fn route() -> Router<AppState> {
    Router::new()
        .merge(start::route("/start"))
        .merge(stop::route("/stop"))
        .merge(restart::route("/restart"))
        .merge(status::route("/status"))
}
