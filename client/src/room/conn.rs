use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, watch, OnceCell};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use futures::StreamExt;
use log::{info, warn};
use uuid::Uuid;

use common::udp::UdpConnection;

use crate::utils::local_addr;
use super::{
    RoomConfig,
    WsConnector,
    WsConnection,
    HEARTBEAT_DURATION,
    Error, Result,
};

fn player_ws_url(addr: &SocketAddr) -> String {
    let uuid = Uuid::now_v7();
    format!("ws://{addr}/{uuid}")
}

#[derive(Debug)]
enum WsSessionSignal {
    HeartbeatTimeout,
    WsDisconnected,
    WsClosed,
    UdpError,
}

#[derive(Debug)]
struct ProxyConnection {
    room: Arc<RoomConfig>,
    handle: JoinHandle<()>,
    udp_conn: Arc<UdpConnection>,      // connection with downstream client
    ws_tx: mpsc::Sender<Message>,      // send data to ws (through internal task)
    
    status: watch::Receiver<ProxyStatus>,
}

impl Drop for ProxyConnection {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl ProxyConnection {
    pub async fn spawn(
        room_cfg: Arc<RoomConfig>,
        proxy_udp_addr: SocketAddr,
    ) -> Result<Self> {
        // >- Create UDP connection
        let udp_conn = UdpConnection::open(local_addr(0), proxy_udp_addr).await
            .map_err(|_| Error::OpenClientUdp { room: room_cfg.as_ref().clone(), addr: proxy_udp_addr })?;
        let udp_conn = Arc::new(udp_conn);
        // -<

        // >- Make WebSocket connector
        let ws_connector = WsConnector::spawn(Arc::clone(&room_cfg));

        // >- Make WebSocket connection
        let (ws_ext_tx, ws_ext_rx) = mpsc::channel::<Message>(64);
        let (status_tx, status_rx) = watch::channel(ProxyStatus::Idle);
        let udp = Arc::clone(&udp_conn);
        let handle = tokio::spawn(async move {
            Self::run_reconnect(ws_connector, udp, ws_ext_rx, status_tx).await;
        });
        // -<

        Ok(Self {
            room: room_cfg,
            handle,
            udp_conn,
            ws_tx: ws_ext_tx,
            status: status_rx,
        })
    }

    pub(crate) fn ws_tx(&self) -> mpsc::Sender<Message> {
        self.ws_tx.clone()
    }
    
    // send to upstream websocket
    pub async fn ws_send(&self, msg: Message) -> Result<()> {
        self.ws_tx.send(msg).await.map_err(|e|
            Error::WsSendFailed { room: self.room.as_ref().clone(), source: e })
    }

    pub(crate) fn udp(&self) -> Arc<UdpConnection> {
        Arc::clone(&self.udp_conn)
    }
    
    // send to downstream udp client
    pub(crate) async fn udp_send(&self, buf: &[u8]) -> Result<()> {
        self.udp_conn.send(buf).await.map_err(|e|
            Error::UdpSendFailed { room: self.room.as_ref().clone(), source: e })
    }
    
    pub fn status(&self) -> watch::Receiver<ProxyStatus> {
        self.status.clone()
    }
    
    pub fn status_now(&self) -> ProxyStatus {
        self.status.borrow().clone()
    }

    pub fn info(&self) -> ProxyConnectionInfo {
        ProxyConnectionInfo {
            udp_local: self.udp().local_addr().ok(),
            udp_client: self.udp().peer_addr().ok(),
            status: self.status_now(),
        }
    }
    
    async fn run_reconnect(
        ws_connector: WsConnector,
        udp: Arc<UdpConnection>,
        mut ws_rx: mpsc::Receiver<Message>,
        status_tx: watch::Sender<ProxyStatus>,
    ) {
        loop {
            info!("Establishing WebSocket connection...");
            let ws_conn = match ws_connector.connect().await {
                Ok(conn) => conn,
                Err(e) => {
                    warn!("Failed to connect to WebSocket server: {}", e);
                    let _ = status_tx.send(ProxyStatus::Terminated);
                    break;
                }
            };

            let _ = status_tx.send(ProxyStatus::Running);
            let signal = Self::run(ws_conn, &udp, &mut ws_rx).await;

            match signal {
                WsSessionSignal::WsClosed => {
                    info!("WebSocket connection gracefully closed");
                    let _ = status_tx.send(ProxyStatus::Terminated);
                    break;
                }
                WsSessionSignal::UdpError => {
                    warn!("UDP error occurred, proxy connection terminated");
                    let _ = status_tx.send(ProxyStatus::Terminated);
                    break;
                }
                WsSessionSignal::HeartbeatTimeout => {
                    warn!("Heartbeat timeout, reconnecting...");
                    let _ = status_tx.send(ProxyStatus::Reconnecting);
                }
                WsSessionSignal::WsDisconnected => {
                    info!("WebSocket disconnected, reconnecting...");
                    let _ = status_tx.send(ProxyStatus::Reconnecting);
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        ws_connector.abort();
    }

    async fn run(
        ws_conn: WsConnection,
        udp_conn: &Arc<UdpConnection>,
        external_rx: &mut mpsc::Receiver<Message>,
    ) -> WsSessionSignal {
        let heartbeat = Arc::new(AtomicU32::new(0));
        let (sig_tx, mut sig_rx) = mpsc::channel(4);

        // Websocket Heartbeat handling
        let ws_tx = ws_conn.tx();
        let sig_tx_ = sig_tx.clone();
        let heartbeat_ = Arc::clone(&heartbeat);
        let heartbeat_task = tokio::spawn(async move {
            let mut heart_tx = heartbeat_.load(Ordering::Relaxed);
            let mut interval = tokio::time::interval(HEARTBEAT_DURATION);

            loop {
                interval.tick().await;
                let heart_rx = heartbeat_.load(Ordering::Relaxed);

                if heart_rx < heart_tx { // last heartbeat was lost
                    let _ = sig_tx_.send(WsSessionSignal::HeartbeatTimeout).await;
                    break;
                }

                heart_tx += 1;
                let payload = heart_tx.to_ne_bytes().to_vec();
                if ws_tx.send(Message::Ping(payload.into())).await.is_err() {
                    let _ = sig_tx_.send(WsSessionSignal::WsDisconnected).await;
                    break;
                }
            }
        });

        // udp data => server ws
        let ws_tx = ws_conn.tx();
        let sig_tx_ = sig_tx.clone();
        let udp = Arc::clone(udp_conn);
        let udp2ws_task = tokio::spawn(async move {
            let mut udp_buf = [0u8; 1500];
            loop {
                match udp.recv(&mut udp_buf).await {
                    Ok(len) => {
                        let text = String::from_utf8_lossy(&udp_buf[..len]).into_owned();
                        if ws_tx.send(Message::Text(text.into())).await.is_err() {
                            let _ = sig_tx_.send(WsSessionSignal::WsDisconnected).await;
                            break;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to recv from UDP: {}", e);
                        let _ = sig_tx_.send(WsSessionSignal::UdpError).await;
                        break;
                    }
                }
            }
        });

        // external ws message forwarding
        let ws_tx = ws_conn.tx();
        let ws_ext_fut = async {
            while let Some(msg) = external_rx.recv().await {
                if ws_tx.send(msg).await.is_err() { break }
            }
        };

        // ws message handling
        let udp = Arc::clone(udp_conn);
        let sig_tx_ = sig_tx.clone();
        let mut ws_rx = ws_conn.rx;
        let heartbeat_ = Arc::clone(&heartbeat);
        let ws2udp_task = tokio::spawn(async move {
            loop {
                match ws_rx.next().await {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(data) => {
                                if let Err(e) = udp.send(data.as_bytes()).await {
                                    warn!("[UDP] Failed to send: {}", e);
                                    let _ = sig_tx_.send(WsSessionSignal::UdpError).await;
                                    break;
                                }
                            }
                            Message::Pong(payload) => {
                                if payload.len() == 4 {
                                    let val = u32::from_ne_bytes([
                                        payload[0], payload[1], payload[2], payload[3]
                                    ]);
                                    heartbeat_.store(val, Ordering::Relaxed);
                                }
                            }
                            Message::Close(_) => {
                                info!("WebSocket received close frame");
                                let _ = sig_tx_.send(WsSessionSignal::WsClosed).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        warn!("[WS] Recv error: {}", e);
                        let _ = sig_tx_.send(WsSessionSignal::WsDisconnected).await;
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended");
                        let _ = sig_tx_.send(WsSessionSignal::WsClosed).await;
                        break;
                    }
                }
            }
        });

        let signal = tokio::select! {
            sig = sig_rx.recv() => {
                sig.unwrap_or(WsSessionSignal::WsDisconnected)
            },
            _ = ws_ext_fut => {
                WsSessionSignal::WsDisconnected
            },
        };

        heartbeat_task.abort();
        udp2ws_task.abort();
        ws2udp_task.abort();

        signal
    }

    pub async fn ws_send_text_buf(&self, buf: &[u8]) -> Result<()> {
        let text = String::from_utf8_lossy(buf).into_owned();
        self.ws_send(Message::Text(text.into())).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyStatus {
    Idle,
    Running,
    Reconnecting,
    Terminated,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        ProxyStatus::Idle
    }
}

#[derive(Debug)]
pub struct ProxyConnectionInfo {
    udp_local: Option<SocketAddr>,
    udp_client: Option<SocketAddr>,
    status: ProxyStatus,
}

#[derive(Debug)]
pub struct LazyProxyConnection {
    room: Arc<RoomConfig>,
    udp_client_addr: SocketAddr,
    conn: OnceCell<ProxyConnection>,
}

impl LazyProxyConnection {
    pub fn new(room_cfg: Arc<RoomConfig>, udp_client_addr: SocketAddr) -> Self {
        LazyProxyConnection {
            room: room_cfg,
            udp_client_addr,
            conn: OnceCell::new(),
        }
    }

    pub async fn spawn(&self) -> Result<()> {
        self.conn.get_or_try_init(|| async {
            ProxyConnection::spawn(Arc::clone(&self.room), self.udp_client_addr).await
        }).await?;
        Ok(())
    }

    pub async fn ws_send_text_buf(&self, buf: &[u8]) -> Result<()> {
        let conn = self.conn.get()
            .ok_or(Error::ProxyNotInitialized { room: self.room.as_ref().clone() })?;
        conn.ws_send_text_buf(buf).await
    }
    
    pub fn status(&self) -> Option<watch::Receiver<ProxyStatus>> {
        self.conn.get().map(|c| c.status())
    }
    
    pub fn status_now(&self) -> Option<ProxyStatus> {
        self.conn.get().map(|c| c.status_now())
    }
    
    pub fn info(&self) -> Option<ProxyConnectionInfo> {
        self.conn.get().map(|c| c.info())
    }
}
