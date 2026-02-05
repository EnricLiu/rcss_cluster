use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use log::{error, info, warn};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use common::client::{Client, Error as ClientError};
use crate::state::AppState;
use crate::PEER_IP;

// Timeout for inactive UDP sessions
const SESSION_TIMEOUT: Duration = Duration::from_secs(60);
// Cleanup interval
const CLEANUP_INTERVAL: Duration = Duration::from_secs(10);
// Default backend port (UDP server port)
pub const DEFAULT_SERVER_UDP_PORT: u16 = 6000;

struct SessionInfo {
    uuid: Uuid,
    client: Arc<Client>,
    last_active: Instant,
    forward_task: JoinHandle<()>,
}

impl Drop for SessionInfo {
    fn drop(&mut self) {
        self.forward_task.abort();
    }
}

pub struct UdpProxy {
    socket: Arc<UdpSocket>,
    sessions: Arc<DashMap<SocketAddr, SessionInfo>>,
    state: AppState,
}

impl UdpProxy {
    pub async fn new(state: AppState, port: u16) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        let socket = Arc::new(socket);
        let sessions =
            Arc::new(DashMap::<SocketAddr, SessionInfo>::new());

        info!("[UDP Proxy] Listening on 0.0.0.0:{}", port);

        // Start cleanup task
        let sessions_clone = sessions.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CLEANUP_INTERVAL);
            loop {
                interval.tick().await;
                let now = Instant::now();

                let mut keys_to_remove = Vec::new();
                for r in sessions_clone.iter() {
                    if now.duration_since(r.value().last_active) > SESSION_TIMEOUT {
                        keys_to_remove.push(*r.key());
                    }
                }

                for key in keys_to_remove {
                    if let Some((_, session)) = sessions_clone.remove(&key) {
                         info!("[UDP Proxy] Session timeout for {}, UUID: {}", key, session.uuid);
                         state_clone.session.remove(&session.uuid);
                    }
                }
            }
        });

        Ok(Self {
            socket,
            sessions,
            state,
        })
    }

    pub async fn run(self) {
        let mut buf = [0u8; 4096];
        loop {
            let (len, addr) = match self.socket.recv_from(&mut buf).await {
                Ok(v) => v,
                Err(e) => {
                    error!("[UDP Proxy] Recv error: {}", e);
                    continue;
                }
            };

            let data_str = match std::str::from_utf8(&buf[..len]) {
                Ok(v) => v,
                Err(_) => {
                     warn!("[UDP Proxy] Received non-UTF8 data from {}, ignoring.", addr);
                     continue;
                }
            };

            if !self.sessions.contains_key(&addr) {
                let uuid = Uuid::now_v7();
                let server_port = self.state.service.config().server.port.unwrap_or(DEFAULT_SERVER_UDP_PORT);
                let server_addr = SocketAddr::new(PEER_IP, server_port);

                let name = Some(format!("udp-{}", addr));
                let client = self.state.session.get_or_create(uuid, name, server_addr);

                let connect_result = client.connect().await;
                match connect_result {
                    Ok(_) => {},
                    Err(ClientError::AlreadyConnected { .. }) => {},
                    Err(e) => {
                         error!("[UDP Proxy] Failed to connect upstream for {}: {}", addr, e);
                         continue;
                    }
                }

                let (tx, mut rx) = mpsc::channel(32);
                let _sub_id = client.subscribe(tx);

                let socket_clone = self.socket.clone();
                let forward_task = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        let bytes = msg.as_bytes();
                        if let Err(_e) = socket_clone.send_to(bytes, addr).await {
                             // quiet error
                        }
                    }
                });

                self.sessions.insert(addr, SessionInfo {
                    uuid,
                    client: client.clone(),
                    last_active: Instant::now(),
                    forward_task,
                });
                info!("[UDP Proxy] New session established for {}", addr);
            }

            if let Some(mut session) = self.sessions.get_mut(&addr) {
                session.last_active = Instant::now();
                if let Err(e) = session.client.send_data(data_str.into()).await {
                    error!("[UDP Proxy] Failed to send data upstream for {}: {}", addr, e);
                }
            }
        }
    }
}
