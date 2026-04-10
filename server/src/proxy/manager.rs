use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Weak};

use dashmap::DashMap;
use log::{debug, info};
use uuid::Uuid;

use common::client::{Client, Info as ClientInfo, Config as ClientConfig};

#[derive(Clone, Default)]
pub struct SessionManager {
    sessions: DashMap<Uuid, Weak<Client>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Retrieve an existing active client or create a new one.
    pub fn get_or_create(
        &self,
        id: Uuid,
        name: Option<String>,
        server_addr: SocketAddr,
    ) -> Arc<Client> {
        // Try to find existing
        if let Some(entry) = self.sessions.get(&id) {
             if let Some(client) = entry.upgrade() {
                 return client;
             }
        }

        // Create new
        let client_config = {
            let mut builder = ClientConfig::builder();
            builder.name = name;
            builder.with_peer(server_addr);
            builder.build_into()
        };

        let client = Arc::new(Client::new(client_config));
        self.sessions.insert(id, Arc::downgrade(&client));
        
        info!("[SessionManager] Created new client session for {}", id);
        
        client
    }

    pub fn remove(&self, id: &Uuid) {
        if self.sessions.remove(id).is_some() {
            debug!("[SessionManager] Removed session reference for {}", id);
        }
    }

    pub fn upgrade(&self, id: &Uuid) -> Option<Arc<Client>> {
        let mut ret = None;
        if let Some(entry) = self.sessions.get(id) {
            ret = entry.upgrade()
        }
        
        if ret.is_none() {
            self.sessions.remove(id);
        }
        
        ret
    }

    fn upgrade_all_iter(&self) -> impl Iterator<Item = (Uuid, Arc<Client>)> + '_ {
        // essential collect here to prevent locking
        let entries: Vec<_> = self.sessions.iter().map(|en| en.key().clone()).collect();

        entries.into_iter()
            .filter_map(|uuid| self.upgrade(&uuid).map(|client| (uuid, client)))
    }
    
    pub fn upgrade_all(&self) -> HashMap<Uuid, Arc<Client>> {
        self.upgrade_all_iter().collect()
    }
    
    pub async fn conn_info(&self) -> HashMap<Uuid, ClientInfo> {
        let clients = self.upgrade_all_iter()
            .map(|(uuid, client)| async move { (uuid, client.info().await) });
        
        futures::future::join_all(clients).await.into_iter().collect()
    }
}
