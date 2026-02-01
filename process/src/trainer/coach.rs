use super::Builder;
use crate::client::{CallResolver, Result, RichClient};
use common::client::RxData;
use common::command::trainer::TrainerCommand;
use common::{client, command};
use log::{debug, error, trace};
use arcstr::ArcStr;
use std::ops::{Deref, DerefMut};
use common::command::{CommandAny};

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
            .map_err(|_| crate::client::Error::ResolverNotSingleton)?;
        let id = self.subscribe(
            resolver.ingest_tx()
                .ok_or(crate::client::Error::ResolverNotSingleton)?
        );
        trace!("[OfflineCoach] CallResolver addon initialized, id = {id}");
        self.addons.insert("call_resolver", Box::new(resolver));

        Ok(())
    }

    pub async fn connect(&self) -> Result<()> {
        trace!(
            "[OfflineCoach] Connecting to host {:?} via peer {:?}",
            self.config().host,
            self.config().peer
        );
        self.conn_connect().await?;
        debug!("[OfflineCoach] Connected.");
        self.init_resolver()?;
        debug!("[OfflineCoach] CallResolver initialized.");

        match self.call(command::trainer::Init { version: Some(5) }).await? {
            Ok(ok) => {
                trace!("[OfflineCoach] Init command succeeded returned with {ok:?}.");
                Ok(())
            },
            Err(e) => {
                error!("[OfflineCoach] Init command returned with error: {}", e);
                Err(crate::client::Error::RcssErrorCall {
                    kind: TrainerCommand::Init.encode(),
                    msg: ArcStr::from(e.to_string())
                })
            },
        }
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
