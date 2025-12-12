use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::future;
use log::{debug, info, warn};
use serde::Serialize;
use tokio::sync::{mpsc, watch, OnceCell};
use tokio::task::JoinHandle;

use common::udp::UdpConnection;
use crate::room::conn::ProxyConnectionInfo;
use super::{
    RoomConfig,
    LazyProxyConnection,
    ProxyStatus,
    Error, Result,
};

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomStatus {
    Running,
    Shutdown(DateTime<Utc>), // stopped at
}

#[derive(Serialize, Debug, Clone)]
pub struct RoomInfo {
    pub config: RoomConfig,
    pub status: RoomStatus,
    pub conn_count: usize,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug)]
pub struct LazyRoom {
    pub cfg: RoomConfig,
    room: OnceCell<Room>,
}

impl LazyRoom {
    pub fn new(cfg: RoomConfig) -> LazyRoom {
        LazyRoom {
            cfg,
            room: OnceCell::new(),
        }
    }

    pub async fn spawn(self: Arc<Self>) -> Result<JoinHandle<()>> {
        let room = self.room.get_or_try_init(||
            async { Room::listen(self.cfg.clone()).await }).await?;
        let status_rx = room.status();

        let task = tokio::spawn(async move {
            let mut status_rx = status_rx;
            while let Ok(_) = status_rx.changed().await {
                if let RoomStatus::Shutdown(_) = *status_rx.borrow() {
                    break;
                }
            }
            drop(self)
        });

        Ok(task)
    }

    pub fn status(&self) -> Option<watch::Receiver<RoomStatus>> {
        self.room.get().map(|room| room.status())
    }

    pub fn status_now(&self) -> Option<RoomStatus> {
        self.room.get().map(|room| room.status_now())
    }

    pub fn uptime(&self) -> Option<Duration> {
        self.room.get().map(|room| room.uptime())
    }
    
    pub fn config(&self) -> Option<RoomConfig> {
        self.room.get().map(|room| room.config().clone())
    }

    pub fn info(&self) -> Option<RoomInfo> {
        self.room.get().map(|room| room.info())
    }
}

#[derive(Debug)]
pub struct Room {
    pub cfg: Arc<RoomConfig>,
    connections: Arc<DashMap<u16, LazyProxyConnection>>,
    udp_listen_task: JoinHandle<()>,
    cleanup_task: JoinHandle<()>,
    status_rx: watch::Receiver<RoomStatus>,
    created_at: DateTime<Utc>,
}

impl Drop for Room {
    fn drop(&mut self) {
        info!("Room[{}] Shutting down", self.cfg.name);
    }
}

impl Room {
    pub async fn listen(mut cfg: RoomConfig) -> Result<Room> {
        let created_at = Utc::now();
        let (status_tx, status_rx) = watch::channel(RoomStatus::Running);

        let udp = UdpConnection::bind(cfg.player_udp).await
            .map_err(|_| Error::OpenRoomUdp { room: cfg.clone() })?;
        let udp_addr = udp.local_addr().expect("Failed to get local udp address");
        cfg.player_udp = udp_addr;
        debug!("Room[{}] UDP listening on {}", cfg.name, udp_addr);

        let cfg = Arc::new(cfg);
        let connections = Arc::new(DashMap::new());

        let (monitor_tx, monitor_rx) = mpsc::channel(32);

        let room_cfg = Arc::clone(&cfg);
        let conn_map_ = Arc::clone(&connections);
        let cleanup_task = tokio::spawn(async move {
            Self::run_cleanup(&room_cfg, conn_map_, monitor_rx).await
        });
        debug!("Room[{}] Cleanup task started", cfg.name);

        let room_cfg = Arc::clone(&cfg);
        let conn_map = Arc::clone(&connections);
        let udp_listen_task = tokio::spawn(async move {
            Self::run_udp_listen(room_cfg, &udp, &monitor_tx, &conn_map).await
        });
        debug!("Room[{}] UDP listening task started", cfg.name);

        Ok(Room {
            cfg,
            udp_listen_task,
            cleanup_task,
            status_rx,

            connections,
            created_at,
        })
    }

    async fn run_udp_listen(
        cfg: Arc<RoomConfig>,
        udp: &UdpConnection,
        monitor_tx: &mpsc::Sender<(u16, watch::Receiver<ProxyStatus>)>,
        connections: &DashMap<u16, LazyProxyConnection>,
    ) {
        let mut buf = [0u8; 1500];
        while let Ok((len, udp_client_addr)) = udp.recv_from(&mut buf).await {
            let config = Arc::clone(&cfg);
            let ident = udp_client_addr.port();

            let mut is_new = false;
            let config_ = Arc::clone(&config);
            connections.entry(ident).or_insert_with(|| {
                is_new = true;
                LazyProxyConnection::new(config_, udp_client_addr)
            });

            let conn = match connections.get(&ident) {
                Some(conn) => conn,
                None => {
                    warn!("Room[{}] Failed to find proxy connection for udp client [{udp_client_addr}]", config.name);
                    continue
                }
            };

            if let Err(e) = conn.spawn().await {
                warn!("Room[{}] Failed to spawn proxy connection: {e}, dropping", config.name);
                connections.remove(&ident);
                continue;
            }

            if is_new {
                let status_rx = conn.status()
                    .expect("Connection not spawned to get status rx.");
                let _ = monitor_tx.send((ident, status_rx)).await;
            }

            if let Err(e) = conn.ws_send_text_buf(&buf[..len]).await {
                info!("Room[{}] Failed to send udp data to proxy connection: {e}, dropping", config.name);
                connections.remove(&ident);
            }
        }
    }

    async fn run_cleanup(
        cfg: &RoomConfig,
        connections: Arc<DashMap<u16, LazyProxyConnection>>,
        mut monitor_rx: mpsc::Receiver<(u16, watch::Receiver<ProxyStatus>)>,
    ) {
        let mut watchers: Vec<(u16, watch::Receiver<ProxyStatus>)> = Vec::new();

        loop {
            if watchers.is_empty() {
                match monitor_rx.recv().await {
                    Some((ident, rx)) => watchers.push((ident, rx)),
                    None => break, // channel closed
                }
                continue;
            }

            tokio::select! {
                new_conn = monitor_rx.recv() => {
                    match new_conn {
                        Some((ident, rx)) => watchers.push((ident, rx)),
                        None => break,
                    }
                }
                _ = Self::wait_any_terminated(&mut watchers) => {
                    let mut i = 0;
                    while i < watchers.len() {
                        let (ident, rx) = &watchers[i];
                        if *rx.borrow() == ProxyStatus::Terminated {
                            info!("Room[{}] Connection {ident} terminated, removing from map", cfg.name);
                            connections.remove(ident);
                            watchers.swap_remove(i);
                        } else {
                            i += 1;
                        }
                    }
                }
            }
        }
    }

    async fn wait_any_terminated(watchers: &mut [(u16, watch::Receiver<ProxyStatus>)]) {
        if watchers.is_empty() {
            future::pending::<()>().await;
            return;
        }

        let futures: Vec<_> = watchers.iter_mut()
            .map(|(_, rx)| {
                Box::pin(async move {
                    loop {
                        if rx.changed().await.is_err() { break }
                        if *rx.borrow() == ProxyStatus::Terminated { break }
                    }
                })
            }).collect();

        let _ = future::select_all(futures).await;
    }

    pub fn info(&self) -> RoomInfo {
        RoomInfo {
            config: self.config().clone(),
            status: self.status_now(),
            conn_count: self.conn_count(),
            created_at: self.created_at,
        }
    }
    
    pub fn conn_infos(&self) -> Vec<ProxyConnectionInfo> {
        self.connections.iter().filter_map(|entry| entry.info()).collect()
    }

    pub fn config(&self) -> &RoomConfig {
        &self.cfg
    }

    pub fn conn_count(&self) -> usize {
        self.connections.len()
    }

    pub fn status(&self) -> watch::Receiver<RoomStatus> {
        self.status_rx.clone()
    }

    pub fn status_now(&self) -> RoomStatus {
        *self.status_rx.borrow()
    }

    pub fn uptime(&self) -> Duration {
        (Utc::now() - self.created_at).to_std().unwrap_or_default()
    }
}
