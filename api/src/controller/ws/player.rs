use std::net::SocketAddr;
use arcstr::ArcStr;
use tokio::sync::mpsc;
use axum::extract::{Path, State, ws::WebSocket, WebSocketUpgrade, Query};
use axum::{routing, Router, response::Response as AxumResponse};
use axum::body::Bytes;
use axum::extract::ws::Message;
use uuid::Uuid;
use serde::Deserialize;

use sidecar::{Service, PEER_IP};
use common::client::{Client, Config as ClientConfig};

use super::AppState;

pub const DEFAULT_SERVER_UDP_PORT: u16 = 6000;

#[derive(Deserialize, Debug)]
struct UpdateRequest {
    name: Option<String>,
    team_name: String,
}
async fn upgrade(
    State(s): State<AppState>,
    ws: WebSocketUpgrade, Path(client_id): Path<Uuid>,
    Query(req): Query<UpdateRequest>
) -> AxumResponse {
    ws.on_upgrade(move |socket| async move {
        handle_upgrade(socket, &s.clone(), client_id, req).await
    })
}

use futures::{SinkExt, StreamExt};
use log::error;

async fn handle_upgrade(socket: WebSocket, state: &AppState, client_id: Uuid, req: UpdateRequest) {
    let (mut socket_tx, mut socket_rx) = socket.split();

    let client_config = {
        let mut builder = ClientConfig::builder();
        builder.name = req.name;
        let server_addr = SocketAddr::new(
            PEER_IP, state.service.config.server.port.unwrap_or(DEFAULT_SERVER_UDP_PORT));
        builder.with_peer(server_addr);

        builder.build_into()
    };


    let player_client = Client::new(client_config);
    let (client_tx, mut client_rx) = mpsc::channel(32);
    player_client.subscribe(client_tx);
    if let Err(e) = player_client.connect().await {
        error!("[Player WS] Client[{client_id}] Failed to connect to server: {}", e);
        let _ = socket_tx.send("Failed to connect to server".into()).await;
        return;
    }

    loop {
        tokio::select! {
            Some(msg) = socket_rx.next() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("[Player WS] Client[{client_id}] Failed to receive message: {}", e);
                        return;
                    },
                };

                match msg {
                    Message::Text(text) => { // treat text as data signals
                        let text = text.trim();
                        if text.is_empty() { continue; }
                    },

                    Message::Binary(bin) => { // treat binary as control signals
                        if let Err(e) = socket_tx.send(Message::Binary(bin)).await {
                            error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
                            return;
                        }
                    },

                    Message::Ping(ping) => {
                        if let Err(e) = socket_tx.send(Message::Pong(ping)).await {
                            error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
                            return;
                        }
                    },

                    _ => {}
                }
            },
            Some(msg) = client_rx.recv() => {
                let message = match ArcStr::as_static(&msg) {
                    Some(text) => Message::Text(text.into()),
                    None => Message::Binary(msg.to_string().into()),
                };

                if let Err(e) = socket_tx.send(message).await {
                    error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
                    return;
                }
            }
        }
    }
}
