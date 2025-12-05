use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::{mpsc, OnceCell};
use dashmap::DashMap;
use arcstr::ArcStr;
use log::{debug, trace};
use tokio::time::error::Elapsed;
use uuid::Uuid;

use common::client;
use common::client::{RxData, TxData};
use super::addon::{Addon, CallerAddon, RawAddon};
use super::command::{self, Command};
use super::resolver::{CallResolver, Sender};
use super::{Error, Result};

pub const DEFAULT_LOCAL_PLAYER_PORT: u16 = 6000;
pub const DEFAULT_LOCAL_TRAINER_PORT: u16 = 6001;

#[derive(Clone, Debug)]
pub struct RichClientBuilder {
    pub conn_builder: client::Builder,
}

impl Default for RichClientBuilder {
    fn default() -> Self {
        let mut conn_builder = client::Builder::new();
        conn_builder
            .with_kind(client::Kind::Player)
            .with_name("Default Player".to_string())
            .with_local_peer(DEFAULT_LOCAL_PLAYER_PORT);

        Self {
            conn_builder
        }
    }
}

impl RichClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_kind(&mut self, kind: client::Kind) -> &mut Self {
        self.conn_builder.with_kind(kind);
        self
    }

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.conn_builder.with_name(name);
        self
    }
    pub fn with_peer(&mut self, peer: SocketAddr) -> &mut Self {
        self.conn_builder.with_peer(peer);
        self
    }
    pub fn with_host(&mut self, host: SocketAddr) -> &mut Self {
        self.conn_builder.with_host(host);
        self
    }
    pub fn with_local_peer(&mut self, port: u16) -> &mut Self {
        self.conn_builder.with_local_peer(port);
        self
    }
    pub fn with_local_host(&mut self, port: u16) -> &mut Self {
        self.conn_builder.with_local_host(port);
        self
    }

    pub fn build(&self) -> RichClient {
        RichClient::from_client_config(self.conn_builder.build())
    }

    pub fn build_into(self) -> RichClient {
        RichClient::from_client_config(self.conn_builder.build_into())
    }
}

#[derive(Debug)]
pub struct RichClient<const BUF_SIZE: usize = 32> {
    conn: client::Client,
    resolver_tx: OnceCell<Sender<TxData, RxData>>,
    addons: DashMap<&'static str, Box<dyn Addon>>,
}

impl<const BUF_SIZE: usize> RichClient<BUF_SIZE> {
    pub fn builder() -> RichClientBuilder {
        RichClientBuilder::default()
    }
    
    pub fn new(
        name: String,
        host: Option<SocketAddr>,
        peer: Option<SocketAddr>
    ) -> Self {
        
        let mut config = client::Config::builder();
        config.with_name(name).with_kind(client::Kind::Trainer);
        config.host = host;
        config.peer = peer;

        Self::from_client_config(config.build_into())
    }
    
    pub fn from_client_config(config: client::Config) -> Self {
        let conn = client::Client::new(config);
        Self { conn, resolver_tx: OnceCell::new(), addons: DashMap::new() }
    }

    fn add_raw_addon<A: RawAddon>(&self, name: &'static str) -> Uuid {
        trace!("[RichClient] Adding raw addon '{name}'");
        let (tx, rx) = mpsc::channel(BUF_SIZE);
        let id = self.conn.subscribe(tx);
        self.addons.insert(name, Box::new(
            A::from_raw(self.conn.signal_sender(), self.conn.data_sender(), rx)
        ));

        trace!("[RichClient] Addon '{name}' added, id = {id}");
        id
    }

    #[must_use]
    pub fn add_caller_addon<A: CallerAddon>(&self, name: &'static str) -> A::Handle {
        trace!("[RichClient] Adding caller-based addon '{name}'");
        let addon = A::from_caller(self.conn.signal_sender(), self.caller());
        let handle = addon.handle();

        self.addons.insert(name, Box::new(addon));
        trace!("[RichClient] Addon '{name}' added");

        handle
    }

    pub(super) fn init_resolver(&self) -> Result<Uuid> {
        trace!("[RichClient] Initializing CallResolver addon.");
        let resolver = CallResolver::new(BUF_SIZE);
        self.resolver_tx.set(resolver.sender(self.conn.data_sender())).unwrap();
        let id = self.subscribe(resolver.ingest_tx().expect("CallResolver is not singleton"));
        trace!("[RichClient] CallResolver addon initialized, id = {id}");
        self.addons.insert("call_resolver", Box::new(resolver));

        Ok(id)
    }

    pub(super) async fn conn_connect(&self) -> Result<()> {
        self.conn.connect().await.expect("Failed to connect"); // todo!()
        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        unreachable!()
    }

    pub fn caller(&self) -> Sender<TxData, RxData> {
        self.resolver_tx.get().expect("CallResolver not initialized").clone()
    }

    pub async fn call<T: Command>(
        &self, cmd: T
    ) -> std::result::Result<std::result::Result<T::Ok, T::Error>, Elapsed> {
        self.resolver_tx.get()
            .expect("CallResolver not initialized")
            .call(cmd).await
    }

    pub fn subscribe(&self, ingest_tx: mpsc::Sender<RxData>) -> Uuid {
        self.conn.subscribe(ingest_tx)
    }

    pub fn unsubscribe(&self, id: Uuid) -> bool {
        self.conn.unsubscribe(id)
    }

    async fn send_cmd(&self, ctrl: impl Command) -> Result<()> {
        self.conn.send_data(ctrl.encode()).await
            .map_err(|e| Error::ClientClosed { source: e })?;
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        self.conn.close().await
            .map_err(|e| Error::ClientCloseFailed { source: e })?;

        for (key, addon) in self.addons.into_iter() {
            addon.close();
            trace!("Addon '{}' closed", key);
        }

        Ok(())
    }
    
    pub fn config(&self) -> &client::Config {
        self.conn.config()
    }
    
    pub fn config_mut(&mut self) -> &mut client::Config {
        self.conn.config_mut()
    }
}

impl Default for RichClient {
    fn default() -> Self {
        Self::builder().build()
    }
}
