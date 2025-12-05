use std::ops::{Deref, DerefMut};
use common::client;
use crate::coach::OfflineCoach;
use super::rich_client::{RichClientBuilder, DEFAULT_LOCAL_TRAINER_PORT};

#[derive(Clone, Debug)]
pub struct OfflineCoachBuilder {
    pub builder: RichClientBuilder,
}

impl Default for OfflineCoachBuilder {
    fn default() -> Self {
        let mut builder = RichClientBuilder::new();
        builder
            .with_kind(client::Kind::Trainer)
            .with_name("Default Offline Coach".to_string())
            .with_local_peer(DEFAULT_LOCAL_TRAINER_PORT);

        Self { builder }
    }
}

impl OfflineCoachBuilder {
    pub fn new() -> Self {
        Self {
            builder: RichClientBuilder::new(),
        }
    }

    pub fn build(&self) -> OfflineCoach {
        OfflineCoach::from_client_config(self.builder.conn_builder.build())
    }

    pub fn build_into(self) -> OfflineCoach {
        OfflineCoach::from_client_config(self.builder.conn_builder.build_into())
    }
}

impl Deref for OfflineCoachBuilder {
    type Target = RichClientBuilder;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

impl DerefMut for OfflineCoachBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.builder
    }
}
