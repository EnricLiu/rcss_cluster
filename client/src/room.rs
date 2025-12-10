use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use futures::channel::oneshot;
use futures::stream::SplitStream;

use log::{info, warn};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::net::TcpStream;

use common::udp::UdpConnection;

use tokio_tungstenite as ws;
use ws::tungstenite::Message;
use ws::{WebSocketStream, MaybeTlsStream};
use uuid::Uuid;
use crate::{Error, Result};
use crate::utils::local_addr;

pub const HEARTBEAT_DURATION: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Room {
    pub udp_port: u16,
    pub ws_addr: SocketAddr,
    udp_listen_task: JoinHandle<()>,
    connections: Arc<DashMap<u16, ProxyConnection>>,
}

impl Room {
    pub async fn listen(udp_port: u16, ws_addr: SocketAddr) -> Result<Self> {
        listen(udp_port, ws_addr).await
    }

    // pub async fn player_proxy(&self, ident: u16) -> ProxyConnection {
    //     let udp = UdpConnection::open()
    // }
}

#[derive(Debug)]
pub struct ProxyConnection {
    handle: JoinHandle<()>,
}
pub async fn listen(udp_port: u16, ws_addr: SocketAddr) -> Result<Room> {
    let udp_addr = local_addr(udp_port);

    let udp = UdpConnection::bind(udp_addr).await
        .map_err(|_| Error::OpenRoomUdp { addr: udp_addr })?;

    let connections = Arc::new(DashMap::new());

    let connections_ = Arc::clone(&connections);
    let udp_listen_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 1500];
        while let Ok((len, peer)) = udp.recv_from(&mut buf).await {
            let ident = peer.port();
            if !connections_.contains_key(&ident) {
                // Create new ProxyConnection
                let connection = ProxyConnection {
                    handle: tokio::spawn(async move {
                        // Handle the connection
                    }),
                };
                connections_.insert(ident, connection);
            }

            // Forward the received data to the corresponding ProxyConnection
            if let Some(connection) = connections_.get(&ident) {
                // Here you would forward the data to the connection
            }
        }
    });

    Ok(Room {
        udp_port,
        ws_addr,
        udp_listen_task,
        connections,
    })
}

fn player_ws_url(addr: &SocketAddr) -> String {
    let uuid = Uuid::now_v7();
    format!("ws://{addr}/{uuid}")
}

pub struct WsConnection {
    ws_tx: mpsc::Sender<Message>,
    ws_rx: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    task: JoinHandle<ws::tungstenite::Result<()>>,
}

impl WsConnection {
    pub fn ws_tx(&self) -> mpsc::Sender<Message> {
        self.ws_tx.clone()
    }

    pub fn close(self) {
        self.task.abort()
    }
}

pub struct WsConnector {
    ws_url: String,
    ws_conn_task: JoinHandle<Result<()>>,
    caller: mpsc::Sender<oneshot::Sender<WsConnection>>,
}

impl WsConnector {
    pub fn spawn(ws_url: String) -> Self {
        let (caller, mut rx)
            = mpsc::channel::<oneshot::Sender<WsConnection>>(4);

        let url = ws_url.clone();
        let ws_conn_task = tokio::spawn(async move {
            while let Some(sender) = rx.recv().await {
                let ws_socket = ws::connect_async(&url).await
                    .map_err(|e| Error::WsConnect { url: url.clone(), source: e })?.0;

                let (ws_tx, mut rx) = mpsc::channel(32);
                let (mut tx, ws_rx) = ws_socket.split();

                // finish when all ws_tx close
                let task = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if let Err(e) = tx.send(msg).await {
                            return Err(e)
                        }
                    }

                    Ok(())
                });

                let ws = WsConnection {
                    task,
                    ws_tx,
                    ws_rx,
                };

                if let Err(_) = sender.send(ws) {
                    warn!("Failed to send WsConnection to caller");
                }
            }

            Ok(())
        });

        Self {
            ws_url,
            ws_conn_task,
            caller,
        }
    }
    pub async fn connect(&self) -> WsConnection {
        let (tx, rx) = oneshot::channel();
        if let Err(_) = self.caller.send(tx).await {
            panic!("WsConnector task has been closed");
        }
        rx.await.expect("WsConnector task has been closed")
    }
}

pub async fn create_conn(udp_port: u16, ws_addr: SocketAddr) -> Result<ProxyConnection> {
    // >- Create UDP connection
    let udp_conn = UdpConnection::open(local_addr(0), local_addr(udp_port)).await
        .map_err(|_| Error::OpenClientUdp { addr: local_addr(udp_port) })?;
    let udp_conn = Arc::new(udp_conn);
    // -<

    // >- Make WebSocket connector
    let ws_url = player_ws_url(&ws_addr);
    let ws_connector = WsConnector::spawn(ws_url.clone());
    // -<

    // shared states
    let heartbeat = Arc::new(AtomicU32::new(0));
    let ws_conn = ws_connector.connect().await;

    // Split ws sockets
    let ws_tx = ws_conn.ws_tx();
    let heartbeat_ = Arc::clone(&heartbeat);
    let heartbeat_task = tokio::spawn(async move {
        let mut heart_tx = heartbeat_.load(Ordering::Relaxed);

        let mut interval = tokio::time::interval(HEARTBEAT_DURATION);
        let res = loop {
            interval.tick().await;
            let heart_rx = heartbeat_.load(Ordering::Relaxed);
            if heart_rx < heart_tx { break None } // last heartbeat was lost
            heart_tx += 1;

            let payload = heart_tx.to_ne_bytes().to_vec();
            if let Err(e) = ws_tx.send(Message::Ping(payload.into())).await {
                break Some(e);
            }
        };
    });

    let ws_tx = ws_conn.ws_tx();
    let udp = Arc::clone(&udp_conn);
    let transmission_task = tokio::spawn(async move {
        let mut udp_buf = [0u8; 1500];

        loop {
            let udp_data = match udp.recv(&mut udp_buf).await {
                Ok(len) => &udp_buf[..len],
                Err(e) => {
                    warn!("[Udp]Failed to recv udp data: {}", e);
                    break;
                }
            };

            let udp_text = String::from_utf8_lossy(udp_data).into_owned();

            if let Err(e) = ws_tx.send(Message::Text(udp_text.into())).await {
                warn!("Failed to send udp data to ws: {}, aborting", e);
                break;
            }
        };
    });

    let udp = Arc::clone(&udp_conn);
    let ws_recv_task = tokio::spawn(async move {
        let mut ws = ws_conn;
        loop {
            let res = loop {
                match ws.ws_rx.next().await {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(data) => {
                                if let Err(e) = udp.send(data.as_bytes()).await {
                                    warn!("Failed to send data to udp: {}, aborting", e);
                                    break None;
                                }
                            },
                            Message::Pong(payload) => {
                                if payload.len() == 4 {
                                    let heartbeat_val = u32::from_ne_bytes(
                                        [payload[0], payload[1], payload[2], payload[3]]
                                    );
                                    heartbeat.store(heartbeat_val, Ordering::Relaxed);
                                }
                            },
                            _ => { continue }
                        }
                    },
                    Some(Err(e)) => break Some(e),
                    None => {
                        info!("WebSocket closed");
                        break None;
                    }
                }
            };

            match res {
                None => { // udp error, no recover
                    warn!("UDP connection error, closing ws recv task");
                    break;
                },

                Some(e) => {
                    info!("WebSocket recv task ended with error: {}", e);

                }
            }
        }
    });

    todo!()
}
