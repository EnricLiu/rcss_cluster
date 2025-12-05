use std::ops::{Deref, DerefMut};
use log::{debug, trace};
use common::client;
use crate::coach::{RichClient, Builder, Result, command};

#[derive(Debug)]
pub struct OfflineCoach {
    client: RichClient,
}

impl OfflineCoach {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn client(&self) -> &RichClient {
        &self.client
    }

    pub fn from_client_config(config: client::Config) -> Self {
        assert_eq!(config.kind, client::Kind::Trainer, "ClientKind::Trainer expected");
        let client = RichClient::from_client_config(config);

        Self { client }
    }

    pub async fn connect(&self) -> Result<()> { // todo!("handle error")
        trace!("[RichClient] Connecting to host {:?} via peer {:?}", self.config().host, self.config().peer);
        self.conn_connect().await.expect("Failed to connect");
        debug!("[RichClient] Connected.");
        let _ = self.init_resolver()?;
        debug!("[RichClient] CallResolver initialized.");
        self.call(command::Init { version: Some(5) }).await
            .expect("Failed to send init signal").unwrap();
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        self.client.shutdown().await
    }
}

impl Deref for OfflineCoach {
    type Target = RichClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for OfflineCoach {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
