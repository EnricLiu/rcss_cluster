mod error;
mod health;
mod response;
mod rooms;

use error::Error;
use response::Response;

use std::sync::Arc;

use crate::proxy::ProxyServer;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response as AxumResponse};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;

#[derive(Clone, Debug)]
pub struct AppState {
    server: Arc<ProxyServer>,
}

async fn fallback_404(State(_state): State<AppState>) -> AxumResponse {
    StatusCode::NOT_FOUND.into_response()
}

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(health::route("/health"))
        .merge(rooms::route("/rooms"))
        .fallback(fallback_404)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A,
    server: Arc<ProxyServer>,
) -> JoinHandle<Result<(), String>> {
    let state = AppState { server };

    let app = route("/", state);
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.map_err(|e| e.to_string()) })
}
