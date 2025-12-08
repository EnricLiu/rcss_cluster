mod error;
mod response;
mod http;
mod ws;

use std::sync::Arc;
use axum::Router;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::task::JoinHandle;

pub use response::Response;

use tower_http::trace::TraceLayer;

use sidecar::Service;

#[derive(Clone)]
pub struct AppState {
    service: Arc<Service>
}

pub async fn listen<A: ToSocketAddrs>(
    addr: A,
) -> JoinHandle<Result<(), String>> {
    let state = AppState { service: Arc::new(Service::new().await), };
    
    let app = Router::new()
        .merge(http::route("/", state.clone()))
        .merge(ws::route("/ws", state))
        .route_layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.map_err(|e| e.to_string())
    })
}
 
