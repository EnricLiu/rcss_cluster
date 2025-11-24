use super::client;
use super::{Signal, Error, Result, Builder};
use std::net::SocketAddr;

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
        self.conn.connect().await.expect(todo!());
        Ok(())
    }

    pub async fn send_ctrl(&mut self, ctrl: Signal) -> Result<()> {
        let ctrl = ctrl.encode();
        self.conn
            .send(client::Signal::Data(ctrl))
            .await
            .map_err(|e| Error::ClientClosed { source: e })?;
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        self.conn.close().await.map_err(|e| Error::ClientCloseFailed { source: e })?;
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
