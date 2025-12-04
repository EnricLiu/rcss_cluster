use std::any::Any;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use arcstr::ArcStr;
use dashmap::DashMap;
use log::debug;

use tokio::sync::{mpsc, oneshot};
use tokio::time::error::Elapsed;
use common::client::{TxData, RxData, TxSignal};
use super::addon::{Addon, RawAddon};
use super::command::{Command, CommandKind};

pub const TIMEOUT: Duration = Duration::from_millis(2000);

#[derive(Clone, Debug)]
pub struct CallResolver {
    rx: Arc<Receiver>,
    rx_ingest: Option<mpsc::Sender<RxData>>, // bind to rx Receiver
}

impl CallResolver {
    pub fn from_rx(receiver: mpsc::Receiver<ArcStr>) -> Self {
        let rx = Arc::new(Receiver::new(receiver));
        Self { rx, rx_ingest: None }
    }

    pub fn from_caller(caller: Sender) -> Self {
        let rx = caller.resolver.clone();
        Self { rx, rx_ingest: None }
    }

    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer);
        let rx = Arc::new(Receiver::new(rx));
        Self { rx, rx_ingest: Some(tx) }
    }

    pub fn ingest_tx(&self) -> Option<mpsc::Sender<ArcStr>> {
        self.rx_ingest.clone()
    }

    pub fn sender(&self, tx: mpsc::Sender<ArcStr>) -> Sender {
        Sender::new(tx, Arc::clone(&self.rx))
    }

    pub fn weak(&self, tx: mpsc::WeakSender<ArcStr>) -> WeakSender {
        WeakSender::new(tx, Arc::clone(&self.rx))
    }

    pub fn close(&self) -> () {
        self.rx.close()
    }
}

impl Addon for CallResolver {
    fn close(&self) {
        CallResolver::close(&self);
    }
}

impl RawAddon for CallResolver {
    fn from_raw(
        _: mpsc::Sender<TxSignal>,
        _: mpsc::Sender<TxData>,
        data_rx: mpsc::Receiver<RxData>
    ) -> Self where Self: Sized {
        Self::from_rx(data_rx)
    }
}

/// Receive from the ArcStr channel,
#[derive(Debug)]
struct Receiver {
    recv_task: tokio::task::JoinHandle<()>,

    queue: Arc<DashMap<
        CommandKind,
        VecDeque<oneshot::Sender<Result<Box<dyn Any + Send>, Box<dyn Any + Send>>>>
    >>,
}

impl Receiver {
    fn new(mut receiver: mpsc::Receiver<RxData>) -> Self {
        let tasks: Arc<DashMap<
            CommandKind,
            VecDeque<
                oneshot::Sender<
                    Result<
                        Box<dyn Any + Send>,
                        Box<dyn Any + Send>
                    >
                >
            >
        >> = Arc::new(DashMap::new());

        let tasks_ = Arc::clone(&tasks);
        let recv_task = tokio::spawn(async move {
            while let Some(raw_msg) = receiver.recv().await {
                let msg = raw_msg.trim().trim_end_matches('\0');
                if msg.is_empty() || !msg.starts_with('(') || !msg.ends_with(')') {
                    debug!("{:?}", msg.chars().take(msg.len()-1));
                    debug!("ignoring peer ret, not matching '(.+)': '{msg}'.");
                    continue;
                }

                let msg = if msg == "(init ok)" { "(ok init)" } else { msg };

                let mut msg = msg[1..msg.len()-1].split(' ');
                let (kind, ret) = match msg.next() {
                    Some("ok") => {
                        if  let Some(kind_str) = msg.next() &&
                            let Some(sig_kind) = CommandKind::decode(kind_str) {
                            let rest: Vec<_> = msg.collect();
                            let ret = sig_kind.parse_ret_ok(&rest);
                            match ret {
                                Some(ok) => (sig_kind, Ok(ok)),
                                None => {
                                    debug!("[CallResolver] Ignore \"ok\" for [{}]: {raw_msg}", sig_kind.encode());
                                    continue;
                                }
                            }
                        } else {
                            debug!("[CallResolver] Ignore \"ok\" for unknown Sig: {raw_msg}");
                            continue;
                        }
                    },
                    Some("error") => {
                        let mut ret = None;

                        let rest: Vec<_> = msg.collect();

                        let map_keys = tasks_.iter()
                            .map(|entry| *entry.key());

                        for sig_kind in map_keys {
                            if let Some(err) = sig_kind.parse_ret_err(&rest) {
                                ret = Some((sig_kind, Err(err)));
                                break;
                            }
                        }

                        match ret {
                            Some(error) => error,
                            None => {
                                debug!("[CallResolver] Ignore \"error\" for unknown Sig: {raw_msg}");
                                continue;
                            }
                        }
                    },
                    _ => {
                        debug!("[CallResolver] Ignore unknown msg: {raw_msg}");
                        continue;
                    }
                };

                if let Some(mut queue) = tasks_.get_mut(&kind) {
                    if let Some(tx) = queue.value_mut().pop_front() {
                        if tx.send(ret).is_err() {
                            debug!("[CallResolver] Failed to send return to caller for [{}]", kind.encode());
                        }
                    }
                }
            }
        });

        Self {
            recv_task,
            queue: tasks,
        }
    }

    fn add_queue(
        &self, signal: CommandKind,
        tx: oneshot::Sender<Result<Box<dyn Any + Send>, Box<dyn Any + Send>>>
    ) {
        self.queue.entry(signal).or_default().push_back(tx)
    }

    pub fn close(&self) -> () {
        self.recv_task.abort();
    }
}

#[derive(Clone, Debug)]
pub struct Sender {
    tx: mpsc::Sender<TxData>,
    resolver: Arc<Receiver>,
}

impl Sender {
    fn new(tx: mpsc::Sender<TxData>, resolver: Arc<Receiver>) -> Self {
        Self { tx, resolver }
    }

    async fn send<T: Command>(&self, sig: T) -> Result<T::Ok, T::Error> {
        let sig_kind = sig.kind();
        let sender = &self.tx;
        sender.send(sig.encode()).await.expect("todo!");

        let (tx, rx) = oneshot::channel();
        self.resolver.add_queue(sig_kind, tx);
        match rx.await.expect("todo!") {
            Ok(ok) => {
                let ok = *ok.downcast::<T::Ok>().expect("todo!");
                Ok(ok)
            }
            Err(err) => {
                let err = *err.downcast::<T::Error>().expect("todo!");
                Err(err)
            }
        }
    }
    
    pub async fn call<T: Command>(&self, sig: T) -> Result<Result<T::Ok, T::Error>, Elapsed> {
        tokio::time::timeout(TIMEOUT, self.send(sig)).await
    }

    pub fn downgrade(&self) -> WeakSender {
        let tx = self.tx.downgrade();
        WeakSender::new(tx, Arc::clone(&self.resolver))
    }
}

#[derive(Clone, Debug)]
pub struct WeakSender {
    tx: mpsc::WeakSender<TxData>,
    resolver: Arc<Receiver>,
}

impl WeakSender {
    fn new(tx: mpsc::WeakSender<TxData>, resolver: Arc<Receiver>) -> Self {
        Self { tx, resolver }
    }

    pub async fn send<T: Command>(&self, sig: T) -> Result<T::Ok, T::Error> {
        let sig_kind = sig.kind();
        let sender = self.tx.upgrade().expect("todo!");
        sender.send(sig.encode()).await.expect("todo!");

        let (tx, rx) = oneshot::channel();
        self.resolver.add_queue(sig_kind, tx);
        match rx.await.expect("todo!") {
            Ok(ok) => {
                let ok = *ok.downcast::<T::Ok>().expect("todo!");
                Ok(ok)
            }
            Err(err) => {
                let err = *err.downcast::<T::Error>().expect("todo!");
                Err(err)
            }
        }
    }

    pub fn upgrade(self) -> Option<Sender> {
        let tx = self.tx.upgrade()?;
        Some(Sender::new(tx, self.resolver))
    }
}