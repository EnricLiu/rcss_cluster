use std::sync::{Arc, OnceLock};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;
use dashmap::DashMap;
use log::{debug, info, trace, warn};

use super::error::*;
use super::{AtomicStatus, StatusKind, Config};
use super::{INIT_MSG_TIMEOUT_MS, BUFFER_SIZE, CHANNEL_CAPACITY};

use crate::udp::UdpConnection;

type ConsumersDashMap = DashMap<Uuid, mpsc::WeakSender<Arc<str>>>;
#[derive(Default, Debug)]
pub struct Client {
    config: Config,
    tx:     OnceLock<mpsc::Sender<Arc<str>>>,
    
    consumers:  Arc<ConsumersDashMap>,

    state:  Arc<AtomicStatus>,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Self { config, ..Default::default() }
    }

    pub async fn conn(&self) -> Result<JoinHandle<Result<()>>> {
        let (tx, sender_rx) = mpsc::channel(CHANNEL_CAPACITY);
        self.tx.set(tx).expect("Client tx OnceLock set failed");

        let consumers = self.consumers.clone();
        let context = Context {
            cfg: self.config.clone(),
            status: self.state.clone(), // todo!(Arc inside the state might confusing, refactor later)
        };

        #[cfg(not(debug_assertions))]
        let handle = tokio::spawn(run(sender_rx, consumers, context));
        #[cfg(debug_assertions)]
        let handle = tokio::spawn(run_debug(sender_rx, consumers, context));

        Ok(handle)
    }

    pub async fn send(&self, data: Arc<str>) -> Result<()> {
        self.tx.get().unwrap().send(data).await
            .map_err(|e| Error::ChannelSend { client_name: self.config.name.clone(), source: e })?;

        Ok(())
    }

    pub async fn subscribe(&self, tx: mpsc::WeakSender<Arc<str>>) -> Result<Uuid> {
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

}

#[derive(Clone)]
struct Context {
    cfg:    Config,
    status:  Arc<AtomicStatus>,
}

async fn run_debug(
    mut sender_rx: mpsc::Receiver<Arc<str>>,
    consumers: Arc<ConsumersDashMap>,
    context: Context,
) -> Result<()> {
    let res = run(sender_rx, consumers, context.clone()).await;
    info!("Client[{}]: connection task ended with result: {:?}", context.cfg.name, res);
    res
}

async fn run(
    mut sender_rx: mpsc::Receiver<Arc<str>>,
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
    let success_cnt = sync_messages(&init_resp, &consumers).await?;
    if success_cnt == 0 {
        warn!("Client[{}]: No consumers to receive init response message.", context.cfg.name);
    }

    listen_and_transmit(sender_rx, Arc::new(udp_conn), consumers, context).await
}

async fn wait_init_msg_from_tx(
    rx: &mut mpsc::Receiver<Arc<str>>,
    context: &Context,
) -> Result<Arc<str>> {
    let msg = tokio::time::timeout(
        Duration::from_millis(INIT_MSG_TIMEOUT_MS), rx.recv(),
    ).await;

    match msg {
        Ok(Some(msg)) => {
            Ok(msg)
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
) -> Result<Arc<str>> {
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
    msg: &Arc<str>, consumers: &ConsumersDashMap
) -> Result<usize> {
    let mut tasks = Vec::with_capacity(consumers.len());

    for consumer in consumers.iter() {
        match consumer.value().upgrade() {
            Some(tx) => tasks.push(async move {
                match tx.send(Arc::clone(msg)).await {
                    Ok(_) => Ok(()),
                    Err(_e) => {
                        warn!("Failed to send message to consumer");
                        Err(())
                    },
                }
            }),
            None => { consumers.remove(consumer.key()); }, // The consumer has been dropped
        };
    }

    let res = futures::future::join_all(tasks).await;

    Ok(res.into_iter().filter(|res| res.is_ok()).count())
}

async fn listen_and_transmit(
    mut rx: mpsc::Receiver<Arc<str>>,
    udp: Arc<UdpConnection>,
    consumers: Arc<ConsumersDashMap>,
    context: Context,
) -> Result<()> {
    let context = Arc::new(context);

    let udp_ = Arc::clone(&udp);
    let context_ = Arc::clone(&context);
    let mut udp_send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            udp_.send(msg.as_bytes()).await
                .map_err(|e| Error::Udp { client_name: context_.cfg.name.clone(), source: e })?;
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

            let cnt = sync_messages(&msg, &consumers).await?;
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
