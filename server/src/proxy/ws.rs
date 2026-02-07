use arcstr::ArcStr;
use axum::extract::ws::Message;
use axum::extract::{Path, Query, State, WebSocketUpgrade, ws::WebSocket};
use axum::{Router, response::Response as AxumResponse, routing};
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use log::{error, info, trace, warn};
use tokio::task::JoinHandle;

use common::client::{Error as ClientError};
use crate::state::AppState;
use crate::PEER_IP;
use crate::metrics::collector::METRICS_COLLECTOR;

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
    METRICS_COLLECTOR.websocket_upgrades_total.inc();
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
    let connection_start = Instant::now();
    METRICS_COLLECTOR.player_connections_total.with_label_values(&["websocket"]).inc();
    METRICS_COLLECTOR.active_player_sessions.with_label_values(&["websocket"]).inc();

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
            METRICS_COLLECTOR.session_reused_total.with_label_values(&["websocket"]).inc();
        }
        Err(e) => {
            warn!(
                "[WS Proxy] Client[{client_id}] Failed to connect to server: {}",
                e
            );
            METRICS_COLLECTOR.proxy_errors_total.with_label_values(&["websocket", "connection_failed"]).inc();
            let _ = socket.send("Failed to connect to server".into()).await;
            METRICS_COLLECTOR.active_player_sessions.with_label_values(&["websocket"]).dec();
            return;
        }
    }

    let (socket_tx, mut socket_rx, mut socket_task) = ws_into_mpsc_tx::<32>(socket);

    loop {
        tokio::select! {
            socket_close = &mut socket_task => {
                match socket_close {
                    Ok(Ok(())) => trace!("[WS Proxy] Client[{client_id}] WebSocket closed normally."),
                    Ok(Err(e)) => {
                        warn!("[WS Proxy] Client[{client_id}] WebSocket closed with error: {e}");
                        METRICS_COLLECTOR.player_disconnections_total.with_label_values(&["websocket", "error"]).inc();
                    },
                    Err(e) => {
                        warn!("[WS Proxy] Client[{client_id}] WebSocket task failed to join: {e}");
                        METRICS_COLLECTOR.player_disconnections_total.with_label_values(&["websocket", "task_error"]).inc();
                    },
                }
                break;
            }
            Some(msg) = socket_rx.next() => {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("[WS Proxy] Client[{client_id}] Failed to receive message: {}", e);
                        METRICS_COLLECTOR.proxy_errors_total.with_label_values(&["websocket", "receive_error"]).inc();
                        return;
                    },
                };

                match msg {
                    Message::Text(text) => {
                        let text = text.trim();
                        if text.is_empty() { continue; }

                        let msg_size = text.len() as f64;
                        METRICS_COLLECTOR.proxy_message_size_bytes.with_label_values(&["websocket", "client_to_server"]).observe(msg_size);
                        METRICS_COLLECTOR.proxy_messages_received.with_label_values(&["websocket", "client_to_server"]).inc();

                        let send_start = Instant::now();
                        if let Err(e) = player_client.send_data(text.into()).await {
                             error!("[WS Proxy] Client[{client_id}] Failed send msg to udp client: {}", e);
                             METRICS_COLLECTOR.proxy_errors_total.with_label_values(&["websocket", "send_error"]).inc();
                        } else {
                            let latency = send_start.elapsed().as_secs_f64();
                            METRICS_COLLECTOR.proxy_message_latency.with_label_values(&["websocket"]).observe(latency);
                            METRICS_COLLECTOR.proxy_messages_sent.with_label_values(&["websocket", "client_to_server"]).inc();
                        }
                    },
                    Message::Binary(bin) => {
                         if let Err(e) = socket_tx.send(Message::Binary(bin)).await {
                            error!("[WS Proxy] Client[{client_id}] Failed to send message: {}", e);
                            METRICS_COLLECTOR.proxy_errors_total.with_label_values(&["websocket", "send_error"]).inc();
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
                let msg_size = msg.len() as f64;
                METRICS_COLLECTOR.proxy_message_size_bytes.with_label_values(&["websocket", "server_to_client"]).observe(msg_size);
                METRICS_COLLECTOR.proxy_messages_received.with_label_values(&["websocket", "server_to_client"]).inc();

                let message = match ArcStr::as_static(&msg) {
                    Some(text) => Message::Text(text.into()),
                    None => Message::Binary(msg.to_string().into()),
                };

                if let Err(e) = socket_tx.send(message).await {
                    error!("[WS Proxy] Client[{client_id}] Failed to send message: {}", e);
                    METRICS_COLLECTOR.proxy_errors_total.with_label_values(&["websocket", "send_error"]).inc();
                    break;
                } else {
                    METRICS_COLLECTOR.proxy_messages_sent.with_label_values(&["websocket", "server_to_client"]).inc();
                }
            }
        }
    }

    // Record connection duration and cleanup
    let duration = connection_start.elapsed().as_secs_f64();
    METRICS_COLLECTOR.player_connection_duration.with_label_values(&["websocket"]).observe(duration);
    METRICS_COLLECTOR.active_player_sessions.with_label_values(&["websocket"]).dec();
    METRICS_COLLECTOR.player_disconnections_total.with_label_values(&["websocket", "normal"]).inc();
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
