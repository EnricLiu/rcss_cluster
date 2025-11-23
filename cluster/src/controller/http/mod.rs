mod room;
mod gateway;
mod team;

use axum::Router;
use axum::response::{IntoResponse, Response as AxumResponse};
use axum::http::StatusCode;
use axum::extract::State;
use super::{AppState, Response};

async fn fallback_404(State(_state): State<AppState>) -> AxumResponse {
    StatusCode::NOT_FOUND.into_response()
}

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(room::route("/room"))
        .merge(team::route("/team"))
        .merge(gateway::route("/gateway"))
        .fallback(fallback_404)
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}