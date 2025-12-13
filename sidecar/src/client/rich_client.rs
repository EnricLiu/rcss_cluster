use std::net::SocketAddr;

use dashmap::DashMap;
use log::trace;
use tokio::sync::{OnceCell, mpsc};
use tokio::time::error::Elapsed;
use uuid::Uuid;

use super::addon::{Addon, CallerAddon, RawAddon};
use super::{CallSender, Error, Result};
use common::client;
use common::client::{RxData, TxData};
use common::command::{Command, CommandAny, CommandResult};

pub const DEFAULT_LOCAL_PLAYER_PORT: u16 = 6000;
pub const DEFAULT_LOCAL_TRAINER_PORT: u16 = 6001;

#[derive(Clone, Debug)]
pub struct RichClientBuilder {
    pub conn_builder: client::Builder,
}

impl Default for RichClientBuilder {
    fn default() -> Self {
        Self::player()
    }
}

impl RichClientBuilder {
    pub fn player() -> Self {
        let mut conn_builder = client::Builder::new();
        conn_builder
            .with_kind(client::Kind::Player)
            .with_name("Default Player".to_string())
            .with_local_peer(DEFAULT_LOCAL_PLAYER_PORT);

        Self { conn_builder }
    }

    pub fn trainer() -> Self {
        let mut conn_builder = client::Builder::new();
        conn_builder
            .with_kind(client::Kind::Trainer)
            .with_name("Default Trainer".to_string())
            .with_local_peer(DEFAULT_LOCAL_TRAINER_PORT);

        Self { conn_builder }
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

    pub fn build<CMD: CommandAny>(&self) -> RichClient<CMD> {
        RichClient::from_client_config(self.conn_builder.build())
    }

    pub fn build_into<CMD: CommandAny>(self) -> RichClient<CMD> {
        RichClient::from_client_config(self.conn_builder.build_into())
    }
}

#[derive(Debug)]
pub struct RichClient<CMD, const BUF_SIZE: usize = 32>
where
    CMD: CommandAny,
{
    pub(crate) conn: client::Client,
    pub(crate) resolver_tx: OnceCell<CallSender<CMD, TxData, RxData>>,
    pub(crate) addons: DashMap<&'static str, Box<dyn Addon>>,
}

impl<CMD, const BUF_SIZE: usize> RichClient<CMD, BUF_SIZE>
where
    CMD: CommandAny,
{
    pub fn builder() -> RichClientBuilder {
        RichClientBuilder::default()
    }

    pub fn new(name: String, host: Option<SocketAddr>, peer: Option<SocketAddr>) -> Self {
        let mut config = client::Config::builder();
        config.with_name(name).with_kind(client::Kind::Trainer);
        config.host = host;
        config.peer = peer;

        Self::from_client_config(config.build_into())
    }

    pub fn from_client_config(config: client::Config) -> Self {
        let conn = client::Client::new(config);
        Self {
            conn,
            resolver_tx: OnceCell::new(),
            addons: DashMap::new(),
        }
    }

    fn add_raw_addon<A: RawAddon>(&self, name: &'static str) -> Uuid {
        trace!("[RichClient] Adding raw addon '{name}'");
        let (tx, rx) = mpsc::channel(BUF_SIZE);
        let id = self.conn.subscribe(tx);
        self.addons.insert(
            name,
            Box::new(A::from_raw(
                self.conn.signal_sender(),
                self.conn.data_sender(),
                rx,
            )),
        );

        trace!("[RichClient] Addon '{name}' added, id = {id}");
        id
    }

    #[must_use]
    pub fn add_caller_addon<A: CallerAddon<CMD>>(&self, name: &'static str) -> A::Handle {
        trace!("[RichClient] Adding caller-based addon '{name}'");
        let addon = A::from_caller(self.conn.signal_sender(), self.caller());
        let handle = addon.handle();

        self.addons.insert(name, Box::new(addon));
        trace!("[RichClient] Addon '{name}' added");

        handle
    }

    pub(crate) async fn conn_connect(&self) -> Result<()> {
        self.conn.connect().await.expect("Failed to connect"); // todo!()
        Ok(())
    }

    pub fn caller(&self) -> CallSender<CMD, TxData, RxData> {
        self.resolver_tx
            .get()
            .expect("CallResolver not initialized")
            .clone()
    }

    pub async fn call<T: Command<Kind = CMD>>(
        &self,
        cmd: T,
    ) -> std::result::Result<CommandResult<T>, Elapsed> {
        self.resolver_tx
            .get()
            .expect("CallResolver not initialized")
            .call(cmd)
            .await
    }

    pub fn subscribe(&self, ingest_tx: mpsc::Sender<RxData>) -> Uuid {
        self.conn.subscribe(ingest_tx)
    }

    pub fn unsubscribe(&self, id: Uuid) -> bool {
        self.conn.unsubscribe(id)
    }

    async fn send_cmd(&self, ctrl: impl Command) -> Result<()> {
        self.conn
            .send_data(ctrl.encode())
            .await
            .map_err(|e| Error::ClientClosed { source: e })?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.conn
            .close()
            .await
            .map_err(|e| Error::ClientCloseFailed { source: e })?;

        for kv in self.addons.iter_mut() {
            let (key, addon) = kv.pair();
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

impl<CMD: CommandAny> Default for RichClient<CMD> {
    fn default() -> Self {
        Self::builder().build()
    }
}
