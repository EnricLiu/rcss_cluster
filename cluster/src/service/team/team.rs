use std::sync::{Arc, Weak};

use uuid::Uuid;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::model::team;
use crate::service::client::{self, Client};
use crate::service::team::{Config, Side, AtomicStatus};

use super::error::{Result};

#[derive(Default, Debug)]
pub struct Team {
    side:       Side,
    config:     Config,
    clients:    RwLock<DashMap<Uuid, Arc<Client>>>,
    status:     Arc<AtomicStatus>,
}

impl Team {
    pub fn new(side: Side, config: Config) -> Self {
        Self {
            side,
            config,
            ..Default::default()
        }
    }

    pub async fn reset(&mut self) -> Result<()> {
        self.clients.write().await.clear();
        self.status.set(team::Status::Idle);
        Ok(())
    }

    pub async fn add_client(&self, id: Uuid, client: Arc<Client>) -> Result<Uuid> {
        todo!()
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }
    
    pub async fn info(&self) -> team::Info {
        team::Info {
            name: self.name().to_string(),
            n_client: self.clients.read().await.len(),
            status: self.status.kind(),
        }
    }

    pub fn is_some(&self) -> bool {
        todo!()
    }
}
