use std::net::SocketAddr;
use common::client;
use super::OfflineCoach;

pub const DEFAULT_LOCAL_TRAINER_PORT: u16 = 6002;

#[derive(Debug)]
pub struct OfflineCoachBuilder {
    conn_builder: client::Builder,
}

impl Default for OfflineCoachBuilder {
    fn default() -> Self {
        let mut conn_builder = client::Builder::new();
        conn_builder
            .with_kind(client::Kind::Trainer)
            .with_name("Default Offline Coach".to_string())
            .with_local_peer(DEFAULT_LOCAL_TRAINER_PORT);

        Self {
            conn_builder
        }
    }
}

impl OfflineCoachBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.conn_builder.with_name(name);
        self
    }
    pub fn with_peer(&mut self, peer: SocketAddr) -> &mut Self {
        self.conn_builder.with_peer(peer);
        self
    }
    pub fn with_host(&mut self, host: SocketAddr) -> &mut Self {
        self.conn_builder.with_host(host);
        self
    }
    pub fn with_local_peer(&mut self, port: u16) -> &mut Self {
        self.conn_builder.with_local_peer(port);
        self
    }
    pub fn with_local_host(&mut self, port: u16) -> &mut Self {
        self.conn_builder.with_local_host(port);
        self
    }

    pub fn build(&self) -> OfflineCoach {
        OfflineCoach::from_client_config(self.conn_builder.build())
    }

    pub fn build_into(self) -> OfflineCoach {
        OfflineCoach::from_client_config(self.conn_builder.build_into())
    }
}

