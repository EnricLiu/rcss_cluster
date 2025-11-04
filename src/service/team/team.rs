use std::sync::Arc;

use uuid::Uuid;
use dashmap::DashMap;
use tokio::sync::RwLock;

use crate::service::team::{Config, Side, Status};
use crate::service::udp::UdpConnection;

#[derive(Default, Debug)]
pub struct Team {
    side:       Side,
    config:     Config,
    clients:    DashMap<Uuid, Arc<UdpConnection>>,
    status:     RwLock<Status>,
}

impl Team {
    pub fn new(side: Side, config: Config) -> Self {
        Self {
            side,
            config,
            ..Default::default()
        }
    }

    pub async fn reset(&mut self) -> Result<(), ()> {
        self.clients.clear();

        *self.status.write().await = Status::Idle;
        Ok(())
    }

    pub async fn add_client(&mut self) -> Result<Uuid, ()> {
        todo!()
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn is_some(&self) -> bool {
        todo!()
    }
}
