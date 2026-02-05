use arcstr::ArcStr;
use axum::extract::ws::Message;
use axum::extract::{Path, Query, State, WebSocketUpgrade, ws::WebSocket};
use axum::{Router, response::Response as AxumResponse, routing};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use uuid::Uuid;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use log::{error, info, trace, warn};
use tokio::task::JoinHandle;

use common::client::{Error as ClientError};
use crate::state::AppState;
use crate::PEER_IP;

pub const DEFAULT_SERVER_UDP_PORT: u16 = 6000;

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .route("/{id}", routing::get(upgrade))
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateRequest {
    name: Option<String>,
}

async fn upgrade(
    State(s): State<AppState>,
    ws: WebSocketUpgrade,
    Path(client_id): Path<Uuid>,
    Query(req): Query<UpdateRequest>,
) -> AxumResponse {
    ws.on_upgrade(
        move |socket| async move { handle_upgrade(socket, &s, client_id, req).await },
    )
}

async fn handle_upgrade(
    mut socket: WebSocket,
    state: &AppState,
    client_id: Uuid,
    req: UpdateRequest,
) {
    let server_addr = SocketAddr::new(
        PEER_IP,
        state
            .service
            .config()
            .server
            .port
            .unwrap_or(DEFAULT_SERVER_UDP_PORT),
    );

    let player_client = state.session.get_or_create(client_id, req.name, server_addr);

    let (client_tx, mut client_rx) = mpsc::channel(32);
    player_client.subscribe(client_tx);

    match player_client.connect().await {
        Ok(_) => {
            trace!("[WS Proxy] Client[{client_id}] Connected to server.");
        }
        Err(ClientError::AlreadyConnected { .. }) => {
            info!(
                "[WS Proxy] Client[{client_id}] Already connected/reusing connection."
            );
        }
        Err(e) => {
            warn!(
                "[WS Proxy] Client[{client_id}] Failed to connect to server: {}",
                e
            );
            let _ = socket.send("Failed to connect to server".into()).await;
            return;
        }
    }

    let (socket_tx, mut socket_rx, mut socket_task) = ws_into_mpsc_tx::<32>(socket);

    loop {
        tokio::select! {
            socket_close = &mut socket_task => {
                match socket_close {
                    Ok(Ok(())) => trace!("[WS Proxy] Client[{client_id}] WebSocket closed normally."),
                    Ok(Err(e)) => warn!("[WS Proxy] Client[{client_id}] WebSocket closed with error: {e}"),
                    Err(e) => warn!("[WS Proxy] Client[{client_id}] WebSocket task failed to join: {e}"),
                }
                break;
            }
            Some(msg) = socket_rx.next() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("[WS Proxy] Client[{client_id}] Failed to receive message: {}", e);
                        return;
                    },
                };

                match msg {
                    Message::Text(text) => {
                        let text = text.trim();
                        if text.is_empty() { continue; }
                        if let Err(e) = player_client.send_data(text.into()).await {
                             error!("[WS Proxy] Client[{client_id}] Failed send msg to udp client: {}", e);
                        }
                    },
                    Message::Binary(bin) => {
                         if let Err(e) = socket_tx.send(Message::Binary(bin)).await {
                            error!("[WS Proxy] Client[{client_id}] Failed to send message: {}", e);
                            break;
                        }
                    },
                    Message::Ping(ping) => {
                        if let Err(e) = socket_tx.send(Message::Pong(ping)).await {
                            error!("[WS Proxy] Client[{client_id}] Failed to send message: {}", e);
                            break;
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
                    error!("[WS Proxy] Client[{client_id}] Failed to send message: {}", e);
                    break;
                }
            }
        }
    }
}

fn ws_into_mpsc_tx<const BUF_SIZE: usize>(
    ws: WebSocket,
) -> (
    mpsc::Sender<Message>,
    SplitStream<WebSocket>,
    JoinHandle<Result<(), axum::Error>>,
) {
    let (tx, rx) = mpsc::channel(BUF_SIZE);
    let (socket_tx, socket_rx) = ws.split();

    let task = tokio::spawn(async move {
        let mut socket_tx = socket_tx;
        let mut rx = rx;

        while let Some(msg) = rx.recv().await {
            socket_tx.send(msg).await?
        }
        Ok(())
    });
    (tx, socket_rx, task)
}
