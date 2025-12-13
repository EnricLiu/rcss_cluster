use super::Builder;
use crate::client::{CallResolver, Result, RichClient};
use common::client::RxData;
use common::command::trainer::TrainerCommand;
use common::{client, command};
use log::{debug, trace};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct OfflineCoach {
    client: RichClient<TrainerCommand>,
}

impl OfflineCoach {
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub fn client(&self) -> &RichClient<TrainerCommand> {
        &self.client
    }

    pub fn from_client_config(config: client::Config) -> Self {
        assert_eq!(
            config.kind,
            client::Kind::Trainer,
            "ClientKind::Trainer expected"
        );
        let client = RichClient::from_client_config(config);

        Self { client }
    }

    pub(super) fn init_resolver(&self) -> Result<()> {
        trace!("[OfflineCoach] Initializing CallResolver addon.");
        let resolver = CallResolver::<TrainerCommand, RxData>::new(32);
        self.resolver_tx
            .set(resolver.sender(self.conn.data_sender()))
            .unwrap();
        let id = self.subscribe(resolver.ingest_tx().expect("CallResolver is not singleton"));
        trace!("[OfflineCoach] CallResolver addon initialized, id = {id}");
        self.addons.insert("call_resolver", Box::new(resolver));

        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        // todo!("handle error")
        trace!(
            "[OfflineCoach] Connecting to host {:?} via peer {:?}",
            self.config().host,
            self.config().peer
        );
        self.conn_connect().await.expect("Failed to connect");
        debug!("[OfflineCoach] Connected.");
        self.init_resolver()?;
        debug!("[OfflineCoach] CallResolver initialized.");
        self.call(command::trainer::Init { version: Some(5) })
            .await
            .expect("Failed to send init signal")
            .unwrap();
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.client.shutdown().await
    }
}

impl Deref for OfflineCoach {
    type Target = RichClient<TrainerCommand>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for OfflineCoach {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
