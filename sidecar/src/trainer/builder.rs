use crate::client::RichClientBuilder;
use crate::trainer::OfflineCoach;
use common::client;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct OfflineCoachBuilder {
    pub builder: RichClientBuilder,
}

impl Default for OfflineCoachBuilder {
    fn default() -> Self {
        let mut builder = RichClientBuilder::trainer();
        Self { builder }
    }
}

impl OfflineCoachBuilder {
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
