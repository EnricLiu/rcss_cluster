use dashmap::DashMap;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use arcstr::ArcStr;
use uuid::Uuid;
use crate::coach::signal::{Signal, SignalKind};
use super::{client, signal};
use super::{Error, Result, Builder};

#[derive(Debug)]
pub struct OfflineCoach {
    conn: client::Client,
}

impl OfflineCoach {
    pub fn builder() -> Builder {
        Builder::default()
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

        let conn = client::Client::new(config.build());
        Self { conn }
    }
    
    pub fn from_client_config(config: client::Config) -> Self {
        assert_eq!(config.kind, client::Kind::Trainer, "ClientKind::Trainer expected");
        let conn = client::Client::new(config);
        Self { conn }
    }

    pub async fn connect(&mut self) -> Result<()> {
        self.conn.connect().await.expect("Failed to connect");
        self.send_ctrl(signal::Init { version: Some(5) }).await.expect("Failed to send init signal");
        Ok(())
    }

    pub fn sender(&self) -> mpsc::WeakSender<client::Signal> {
        self.conn.sender()
    }

    pub fn subscribe(&self, tx: mpsc::Sender<ArcStr>) -> Uuid {
        self.conn.subscribe(tx)
    }

    pub fn unsubscribe(&self, id: Uuid) -> bool {
        self.conn.unsubscribe(id)
    }

    pub async fn send_ctrl(&self, ctrl: impl Signal) -> Result<()> {
        let ctrl = ctrl.encode();
        self.conn
            .send(client::Signal::Data(ctrl)).await
            .map_err(|e| Error::ClientClosed { source: e })?;
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        self.conn.close().await
            .map_err(|e| Error::ClientCloseFailed { source: e })?;
        Ok(())
    }
    
    pub fn config_mut(&mut self) -> &mut client::Config {
        self.conn.config_mut()
    }
}

impl Default for OfflineCoach {
    fn default() -> Self {
        Self::builder().build()
    }
}
