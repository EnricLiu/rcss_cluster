use std::net::SocketAddr;
use std::sync::Arc;
use arcstr::ArcStr;
use tokio::sync::mpsc;
use axum::extract::{Path, State, ws::WebSocket, WebSocketUpgrade, Query};
use axum::{routing, Router, response::Response as AxumResponse};
use axum::extract::ws::Message;
use uuid::Uuid;
use serde::Deserialize;

use sidecar::{PEER_IP};
use common::client::{Client, Config as ClientConfig, Error as ClientError};

use super::AppState;

pub const DEFAULT_SERVER_UDP_PORT: u16 = 6000;

#[derive(Deserialize, Debug)]
struct UpdateRequest {
    name: Option<String>,
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
use futures::stream::SplitStream;
use log::{debug, error, info, trace, warn};
use tokio::task::JoinHandle;

async fn handle_upgrade(mut socket: WebSocket, state: &AppState, client_id: Uuid, req: UpdateRequest) {
    let client_config = {
        let mut builder = ClientConfig::builder();
        builder.name = req.name;
        let server_addr = SocketAddr::new(
            PEER_IP, state.sidecar.config().server.port.unwrap_or(DEFAULT_SERVER_UDP_PORT));
        builder.with_peer(server_addr);

        builder.build_into()
    };

    let player_client = {
        let mut client = None;

        state.players.entry(client_id)
            .and_modify(|c| { // existing weak
                client = c.upgrade();
                if client.is_none() {
                    client = Some(Arc::new(Client::new(client_config.clone())));
                    *c = Arc::downgrade(client.as_ref().unwrap());
                }
            })
            .or_insert_with(|| { // create
                client = Some(Arc::new(Client::new(client_config.clone())));
                Arc::downgrade(client.as_ref().unwrap())
            });

       client.unwrap()
    };

    let (client_tx, mut client_rx) = mpsc::channel(32);
    player_client.subscribe(client_tx);

    match player_client.connect().await {
        Ok(_) => {
            trace!("[Player WS] Client[{client_id}] Connected to server.");
        },
        Err(ClientError::AlreadyConnected { .. }) => {
            info!("[Player WS] Client[{client_id}] Already connected, may because of reconnection or broadcasting.");
        },
        Err(e) => {
            warn!("[Player WS] Client[{client_id}] Failed to connect to server: {}", e);
            let _ = socket.send("Failed to connect to server".into()).await;
            return;
        }
    }

    let (
        socket_tx,
        mut socket_rx,
        mut socket_task
    ) = ws_into_mpsc_tx::<32>(socket);

    loop {
        tokio::select! {
            // if the socket task finishes, the ws socket is considered to be closed
            socket_close = &mut socket_task => {
                match socket_close {
                    Ok(Ok(())) => {
                        trace!("[Player WS] Client[{client_id}] WebSocket closed normally.");
                    },
                    Ok(Err(e)) => {
                        warn!("[Player WS] Client[{client_id}] WebSocket closed with error: {e}");
                    },
                    Err(e) => {
                        warn!("[Player WS] Client[{client_id}] WebSocket task failed to join: {e}");
                    }
                }

                break;
            }
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
                        player_client.send_data(text.into()).await.expect("Failed send msg to udp client");
                    },

                    Message::Binary(bin) => { // treat binary as control signals
                        if let Err(e) = socket_tx.send(Message::Binary(bin)).await {
                            error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
                            break;
                        }
                    },

                    Message::Ping(ping) => {
                        if let Err(e) = socket_tx.send(Message::Pong(ping)).await {
                            error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
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
                    error!("[Player WS] Client[{client_id}] Failed to send message: {}", e);
                    break;
                }
            }
        }
    }

    // if is the last reference, close the connection
    if let Some(mut client) = Arc::into_inner(player_client) {
        if let Err(e) = client.close().await {
            error!("[Player WS] Client[{client_id}] Failed to close client: {e}");
        } else {
            trace!("[Player WS] Client[{client_id}] closed normally.");
        }
    }
    // elsewise return directly
}

fn ws_into_mpsc_tx<const BUF_SIZE: usize>(
    ws: WebSocket
) -> (mpsc::Sender<Message>, SplitStream<WebSocket>, JoinHandle<Result<(), axum::Error>>) {
    let (tx, rx) = mpsc::channel(BUF_SIZE);
    let (socket_tx, socket_rx) = ws.split();

    let task = tokio::spawn(async move {
        let mut socket_tx = socket_tx;
        let mut rx = rx;

        while let Some(msg) = rx.recv().await {
            if let Err(e) = socket_tx.send(msg).await {
                return Err(e)
            }
        }

        Ok(())
    });

    (tx, socket_rx, task)
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new()
        .route("/{client_id}", routing::get(upgrade));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
