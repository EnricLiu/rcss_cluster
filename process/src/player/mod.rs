use std::ops::{Deref, DerefMut};
use log::trace;
use common::client::RxData;
use common::command::player::PlayerCommand;
use common::command::trainer::TrainerCommand;
use crate::client::{CallResolver, RichClient, Result};

pub struct Player {
    pub client: RichClient<PlayerCommand>,
}

impl Deref for Player {
    type Target = RichClient<PlayerCommand>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl DerefMut for Player {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl Player {
    pub(super) fn init_resolver(&self) -> Result<()> {
        trace!("[Player] Initializing CallResolver addon.");
        let resolver = CallResolver::<PlayerCommand, RxData>::new(32);
        self.resolver_tx
            .set(resolver.sender(self.conn.data_sender()))
            .map_err(|_| crate::client::Error::ResolverNotSingleton)?;
        let id = self.subscribe(
            resolver.ingest_tx()
                .ok_or(crate::client::Error::ResolverNotSingleton)?
        );
        trace!("[Player] CallResolver addon initialized, id = {id}");
        self.addons.insert("call_resolver", Box::new(resolver));

        Ok(())
    }
}