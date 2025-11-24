use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use super::kind::ClientKind;

static DEFAULT_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);
static DEFAULT_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6000);


#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub name: String,
    pub kind: ClientKind,
    pub host: SocketAddr,
    pub peer: SocketAddr,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            name: "Default Client".to_string(),
            kind: ClientKind::default(),
            host: DEFAULT_HOST,
            peer: DEFAULT_PEER,
        }
    }
}

impl ClientConfig {
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }
}

#[derive(Default, Clone, Debug)]
pub struct ClientConfigBuilder {
    pub name: Option<String>,
    pub kind: Option<ClientKind>,
    pub host: Option<SocketAddr>,
    pub peer: Option<SocketAddr>,
}

impl ClientConfigBuilder {
    pub fn new() -> Self {
        ClientConfigBuilder::default()
    }

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn with_kind(&mut self, kind: ClientKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_host(&mut self, host: SocketAddr) -> &mut Self {
        self.host = Some(host);
        self
    }
    pub fn with_peer(&mut self, peer: SocketAddr) -> &mut Self {
        self.peer = Some(peer);
        self
    }
    
    pub fn with_local_host(&mut self, port: u16) -> &mut Self {
        self.with_host(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
    }
    
    pub fn with_local_peer(&mut self, port: u16) -> &mut Self {
        self.with_peer(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
    }

    pub fn build_into(self) -> ClientConfig {
        let mut config = ClientConfig::default();
        if let Some(name) = self.name { config.name = name };
        if let Some(kind) = self.kind { config.kind = kind };
        if let Some(host) = self.host { config.host = host };
        if let Some(peer) = self.peer { config.peer = peer };

        config
    }
    
    pub fn build(&self) -> ClientConfig {
        self.clone().build_into()
    }
}
