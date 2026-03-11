mod command;
mod control;
mod gateway;
mod health;

use crate::AppState;
use crate::error::Error;
use crate::response::Response;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};

async fn fallback_404(State(_state): State<AppState>) -> AxumResponse {
    StatusCode::NOT_FOUND.into_response()
}

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(command::route("/"))
        .merge(control::route("/control"))
        .merge(gateway::route("/gateway"))
        .fallback(fallback_404)
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
