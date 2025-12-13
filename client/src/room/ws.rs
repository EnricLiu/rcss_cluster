use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use log::{info, warn};
use tokio_tungstenite as ws;
use ws::tungstenite::Message;
use ws::tungstenite::Result as WsResult;
use ws::{MaybeTlsStream, WebSocketStream};

use super::{Error, Result, RoomConfig, WsConfig};

#[derive(Debug)]
pub struct WsConnection {
    pub(crate) tx: mpsc::Sender<Message>,
    pub(crate) rx: futures::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    task: JoinHandle<WsResult<()>>,
}

impl WsConnection {
    pub fn tx(&self) -> mpsc::Sender<Message> {
        self.tx.clone()
    }
    pub fn close(self) {
        self.task.abort()
    }
}

#[derive(Debug)]
pub struct WsConnector {
    pub room: Arc<RoomConfig>,
    ws_conn_task: JoinHandle<Result<()>>,
    caller: mpsc::Sender<oneshot::Sender<Result<WsConnection>>>,
}

impl WsConnector {
    pub fn spawn(room_cfg: Arc<RoomConfig>) -> Self {
        let (caller, mut rx) = mpsc::channel::<oneshot::Sender<_>>(4);

        let room_cfg_ = Arc::clone(&room_cfg);
        let ws_conn_task = tokio::spawn(async move {
            while let Some(sender) = rx.recv().await {
                let ws_socket = match Self::connect_with_retry(&room_cfg_.ws).await {
                    Ok(socket) => socket,
                    Err(e) => {
                        warn!(
                            "Room[{}] Failed to connect to WebSocket after max retries: {}, dropping request",
                            room_cfg_.name, &e
                        );
                        let err = Error::WsConnect {
                            room: room_cfg_.as_ref().clone(),
                            source: e,
                        };

                        let _ = sender.send(Err(err));
                        continue;
                    }
                };

                let (ws_tx, mut rx) = mpsc::channel(32);
                let (mut tx, ws_rx) = ws_socket.split();

                // finish when all ws_tx close
                let task = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if let Err(e) = tx.send(msg).await {
                            return Err(e);
                        }
                    }

                    Ok(())
                });

                let ws = WsConnection {
                    task,
                    tx: ws_tx,
                    rx: ws_rx,
                };

                if let Err(_) = sender.send(Ok(ws)) {
                    warn!(
                        "Room[{}] Failed to send WsConnection to caller",
                        room_cfg_.name
                    );
                }
            }

            Ok::<(), Error>(())
        });

        Self {
            room: room_cfg,
            ws_conn_task,
            caller,
        }
    }

    async fn connect_with_retry(
        config: &WsConfig,
    ) -> std::result::Result<WebSocketStream<MaybeTlsStream<TcpStream>>, ws::tungstenite::Error>
    {
        let mut last_err = None;

        for attempt in 1..=config.max_reconnect_attempts {
            match ws::connect_async(config.base_url.to_string()).await {
                Ok((socket, _)) => {
                    if attempt > 1 {
                        info!("WebSocket connected after {} attempts", attempt);
                    }
                    return Ok(socket);
                }
                Err(e) => {
                    warn!(
                        "WebSocket connect attempt {}/{} failed: {}",
                        attempt, config.max_reconnect_attempts, e
                    );
                    last_err = Some(e);
                    if attempt < config.max_reconnect_attempts {
                        tokio::time::sleep(config.reconnect_delay).await;
                    }
                }
            }
        }

        Err(last_err.expect("WTF? last_err is always set"))
    }

    pub async fn connect(&self) -> Result<WsConnection> {
        let (tx, rx) = oneshot::channel();
        self.caller
            .send(tx)
            .await
            .map_err(|_| Error::WsConnectorDown {
                room: self.room.as_ref().clone(),
            })?;
        rx.await
            .map_err(|_| Error::WsConnectorDown {
                room: self.room.as_ref().clone(),
            })
            .flatten()
    }

    pub fn abort(self) {
        self.ws_conn_task.abort()
    }
}
