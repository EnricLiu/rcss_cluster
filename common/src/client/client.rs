use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;
use arcstr::ArcStr;
use dashmap::DashMap;
use futures::FutureExt;
use log::{debug, info, trace, warn};
use super::error::*;
use super::{AtomicStatus, StatusKind, Config, Signal};
use super::{INIT_MSG_TIMEOUT_MS, BUFFER_SIZE, CHANNEL_CAPACITY};

use crate::udp::UdpConnection;

pub type ClientBuilder = super::config::ClientConfigBuilder;
type ConsumersDashMap = DashMap<Uuid, mpsc::Sender<ArcStr>>;
#[derive(Default, Debug)]
pub struct Client {
    config: Config,
    handle: OnceLock<JoinHandle<Result<()>>>,
    tx:     OnceLock<mpsc::Sender<Signal>>,
    status: Arc<AtomicStatus>,

    consumers:  Arc<ConsumersDashMap>,
}

impl Client {
    pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn new(config: Config) -> Self {
        Self { config, ..Default::default() }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let (tx, sender_rx) = mpsc::channel(CHANNEL_CAPACITY);
        if let Err(_) = self.tx.set(tx) {
            return Err(Error::AlreadyConnected {
                client_name: self.config.name.clone(),
            });
        }

        let consumers = self.consumers.clone();
        let context = Context {
            cfg: self.config.clone(),
            status: self.status.clone(), // todo!(Arc inside the status might confusing, refactor later)
        };

        if self.handle.get().is_some() {
            return Err(Error::AlreadyConnected {
                client_name: self.config.name.clone(),
            });
        }

        #[cfg(not(debug_assertions))]
        let handle = tokio::spawn(run(sender_rx, consumers, context));
        #[cfg(debug_assertions)]
        let handle = tokio::spawn(run_debug(sender_rx, consumers, context));
        if let Err(handle) = self.handle.set(handle) {
            handle.abort();
            return Err(Error::AlreadyConnected {
                client_name: self.config.name.clone(),
            });
        }

        Ok(())
    }

    pub async fn send(&self, signal: Signal) -> Result<()> {
        self.tx.get().unwrap().send(signal).await
            .map_err(|e| Error::ChannelSend { client_name: self.config.name.clone(), source: e })?;

        Ok(())
    }

    pub async fn subscribe(&self, tx: mpsc::Sender<ArcStr>) -> Result<Uuid> {
        let id = Uuid::now_v7();
        self.consumers.insert(id, tx);
        Ok(id)
    }

    pub async fn unsubscribe(&self, id: Uuid) -> bool {
        self.consumers.remove(&id).is_some()
    }

    pub fn name(&self) -> &str {
        self.config.name.as_str()
    }

    pub async fn close(mut self) -> Result<()> {
        if self.handle.get().is_none() {
            return Err(Error::NotConnected { client: self });
        }

        let mut handle = self.handle.take()
            .expect("WTF? Client handle OnceLock get failed");

        if let Err(_e) = self.send(Signal::Shutdown).await {
            // channel closed here, maybe already closed
            info!("Client[{}]: channel closed while trying to send Signal::Shutdown", self.name());
        }

        match tokio::time::timeout(Self::SHUTDOWN_TIMEOUT, &mut handle).await {
            Err(_) => { // Timeout Elapsed
                warn!("Client[{}]: timeout while waiting for shutdown returns, aborting", self.name());
                handle.abort();
                self.status.set(StatusKind::Died);
                Err(Error::CloseTimeout {
                    duration: Self::SHUTDOWN_TIMEOUT,
                    client_name: self.name().to_string(),
                })
            },
            Ok(Err(join)) => {
                if join.is_cancelled() {
                    debug!("Client[{}]: Failed to join: Task already cancelled.", self.name());
                    self.status.set(StatusKind::Disconnected);
                    Ok(())
                } else {
                    warn!("Client[{}]: Failed to join: Task panicked.", self.name());
                    self.status.set(StatusKind::Died);
                    Err(Error::ClosePanic {
                        client_name: self.name().to_string(),
                    })
                }
            },
            Ok(Ok(res)) => {
                self.status.set(StatusKind::Disconnected);
                res
            },
        }
    }

    pub fn config_then(&mut self, f: impl FnOnce(&mut Config)) -> Result<()> {
        if self.status().is_running() {
            return Err(Error::AlreadyConnected {
                client_name: self.config.name.clone(),
            });
        }

        f(&mut self.config);
        Ok(())
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn status(&self) -> StatusKind {
        self.status.kind()
    }
}

#[derive(Clone)]
struct Context {
    cfg:    Config,
    status:  Arc<AtomicStatus>,
}

async fn run_debug(
    sender_rx: mpsc::Receiver<Signal>,
    consumers: Arc<ConsumersDashMap>,
    context: Context,
) -> Result<()> {
    let res = run(sender_rx, consumers, context.clone()).await;
    info!("Client[{}]: connection task ended with result: {:?}", context.cfg.name, res);
    res
}

async fn run(
    mut sender_rx: mpsc::Receiver<Signal>,
    consumers: Arc<ConsumersDashMap>,
    context: Context,
) -> Result<()> {
    assert_eq!(*context.status, StatusKind::Disconnected); // todo!()
    debug!("Client[{}]: starting connection...", context.cfg.name);
    trace!("Client[{}]: Waiting for init msg from tx.", context.cfg.name);

    context.status.set(StatusKind::Idle);
    let init_msg = wait_init_msg_from_tx(&mut sender_rx, &context).await?;
    trace!("Client[{}]: received init msg from tx: {}", context.cfg.name, init_msg);

    context.status.set(StatusKind::WaitingRedirection);
    trace!("Client[{}]: opening UDP connection to {}...", context.cfg.name, context.cfg.peer);
    let mut udp_conn = UdpConnection::open(context.cfg.host, context.cfg.peer).await
        .map_err(|e| Error::Udp { client_name: context.cfg.name.clone(), source: e })?;
    trace!("Client[{}]: UDP connection opened.", context.cfg.name);

    let init_resp = wait_init_resp_recv(&init_msg, &mut udp_conn, &context).await?;
    trace!("Client[{}]: received init resp from server: {}", context.cfg.name, init_resp);
    let success_cnt = sync_messages(&init_resp, &consumers, &context).await?;
    if success_cnt == 0 {
        warn!("Client[{}]: No consumers to receive init response message.", context.cfg.name);
    }

    listen_and_transmit(sender_rx, Arc::new(udp_conn), consumers, context).await
}

async fn wait_init_msg_from_tx(
    rx: &mut mpsc::Receiver<Signal>,
    context: &Context,
) -> Result<ArcStr> {
    let msg = tokio::time::timeout(
        Duration::from_millis(INIT_MSG_TIMEOUT_MS), rx.recv(),
    ).await;

    match msg {
        Ok(Some(Signal::Data(msg))) => Ok(msg),
        Ok(Some(Signal::Shutdown)) => {
            context.status.set(StatusKind::Disconnected);
            Err(Error::ChannelClosed {
                client_name: context.cfg.name.clone()
            })
        },
        Ok(None) => { // Channel closed
            context.status.set(StatusKind::Disconnected);
            Err(Error::ChannelClosed {
                client_name: context.cfg.name.clone()
            })
        },
        Err(_elapsed) => { // Timeout
            context.status.set(StatusKind::Disconnected);
            Err(Error::TimeoutInitReq {
                client_name: context.cfg.name.clone(),
                duration_s: INIT_MSG_TIMEOUT_MS as f32 / 1000.0,
            })
        },
    }
}

async fn wait_init_resp_recv(
    init_msg: &str,
    udp_conn: &mut UdpConnection,
    context: &Context,
) -> Result<ArcStr> {
    let mut buf = [0u8; BUFFER_SIZE];

    udp_conn.send(init_msg.as_bytes()).await
        .map_err(|e| Error::Udp { client_name: context.cfg.name.clone(), source: e })?;

    let recv_result = tokio::time::timeout(
        Duration::from_millis(INIT_MSG_TIMEOUT_MS),
        udp_conn.recv_set_peer(&mut buf),
    ).await;

    match recv_result {
        Ok(Ok(len)) => {
            Ok(String::from_utf8_lossy(&buf[..len]).to_string().into_boxed_str().into())
        },
        Ok(Err(e)) => {
            context.status.set(StatusKind::Disconnected);
            Err(Error::Udp {
                client_name: context.cfg.name.clone(),
                source: e,
            })
        },
        Err(_elapsed) => { // Timeout
            context.status.set(StatusKind::Disconnected);
            Err(Error::TimeoutInitResp {
                client_name: context.cfg.name.clone(),
                duration_s: INIT_MSG_TIMEOUT_MS as f32 / 1000.0,
            })
        },
    }
}

async fn sync_messages(
    msg: &ArcStr, consumers: &ConsumersDashMap, context: &Context,
) -> Result<usize> {
    let mut tasks = Vec::with_capacity(consumers.len());

    for consumer in consumers.iter() {
        tasks.push(async move {
            let (id, tx) = consumer.pair();
            match tx.send(msg.clone()).await {
                Err(_e) => {
                    trace!("Channel[{id}] Closed, winding up..");
                    Some(*id)
                },
                Ok(_) => None,
            }
        });
    }

    let res = futures::future::join_all(tasks).await;
    let mut success_cnt = res.len();

    let deleted = res.into_iter().filter_map(|res| res);
    for id in deleted {
        success_cnt -= 1;
        if let None = consumers.remove(&id) {
            warn!("Client[{}]: Consumer[{}] was removed from the list, but it was not found in the list.",
                context.cfg.name, id);
        }
    }

    Ok(success_cnt)
}

async fn listen_and_transmit(
    mut rx: mpsc::Receiver<Signal>,
    udp: Arc<UdpConnection>,
    consumers: Arc<ConsumersDashMap>,
    context: Context,
) -> Result<()> {
    let context = Arc::new(context);

    let udp_ = Arc::clone(&udp);
    let context_ = Arc::clone(&context);
    let mut udp_send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Signal::Shutdown => break,
                Signal::Data(msg) => {
                    udp_.send(msg.as_bytes()).await
                        .map_err(|e| Error::Udp { client_name: context_.cfg.name.clone(), source: e })?;
                }
            }
        }
        Ok::<(), Error>(())
    });

    let context_ = Arc::clone(&context);
    let mut udp_recv_task = tokio::spawn(async move {
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            let len = udp.recv(&mut buf).await
                .map_err(|e| Error::Udp { client_name: context_.cfg.name.clone(), source: e })?;

            let msg = String::from_utf8_lossy(&buf[..len])
                .to_string().into_boxed_str().into();

            let cnt = sync_messages(&msg, &consumers, &context_).await?;
            if cnt == 0 {
                warn!("Client[{}]: No consumers to receive UDP message.", context_.cfg.name);
            }
        }

        Ok::<(), Error>(())
    });

    let (task_res, task_name) = tokio::select! {
        res = &mut udp_send_task => (res, "listen_and_transmit::udp_send_task"),
        res = &mut udp_recv_task => (res, "listen_and_transmit::udp_recv_task"),
    };

    udp_send_task.abort();
    udp_recv_task.abort();
    context.status.set(StatusKind::Disconnected);
    debug!("Client[{}]: {} ended, shutting down connection.", context.cfg.name, task_name);


    task_res.map_err(|e| Error::TaskJoin {
        client_name: context.cfg.name.clone(),
        task_desc: task_name.to_string(),
        source: e,
    })?
}
