use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::{Child, Command};
use common::types::Side;

use crate::config::{MatchComposerConfig, ServerConfig};
use crate::policy::PolicyRegistry;
use crate::team::Team;

#[derive(Debug, Clone)]
pub struct AgentConnectionInfo {
    pub side: Side,
    pub unum: u8,
    pub team_name: String,
    pub grpc_host: Ipv4Addr,
    pub grpc_port: u16,
}

pub struct MatchComposer {
    pub config: MatchComposerConfig,
    pub log_root: Option<PathBuf>,

    registry: PolicyRegistry,
    server_process: Option<Child>,

    pub allies: Team,
    pub opponents: Team,
}

impl MatchComposer {
    pub fn new(config: MatchComposerConfig, registry_path: impl AsRef<Path>) -> Result<Self, String> {
        let registry = PolicyRegistry::new(registry_path);
        println!("{:?}", registry.images.providers().and_then(|i| Some(i.collect::<Vec<_>>())));

        let allies = Team::new(config.allies.clone(), Side::LEFT);
        let opponents = Team::new(config.opponents.clone(), Side::RIGHT);
        let log_root = config.log_root.clone();
        Ok(Self {
            config,
            registry,
            server_process: None,
            allies,
            opponents,
            log_root,
        })
    }
    
    pub async fn start_server(&mut self) -> Result<(), std::io::Error> {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "server", "--features", "standalone", "--"])
           .arg("--port").arg(self.config.server.port.to_string())
           .arg("--ip").arg(self.config.server.host.to_string());
        
        self.server_process = Some(cmd.spawn()?);
        tokio::time::sleep(Duration::from_secs(2)).await;
        Ok(())
    }
    
    pub async fn shutdown(&mut self) {
        self.allies.shutdown().await;
        self.opponents.shutdown().await;
        if let Some(mut proc) = self.server_process.take() {
            let _ = proc.kill().await;
        }
    }

    pub async fn spawn_players(&mut self) -> Result<(), String> {
        self.allies.spawn(&self.registry).await?;
        self.opponents.spawn(&self.registry).await?;

        Ok(())
    }

    pub async fn wait(&mut self) -> Result<(), String> {
        self.allies.wait().await?;
        self.opponents.wait().await?;
        Ok(())
    }

    pub fn agent_conns(&self) -> Vec<AgentConnectionInfo> {
        let mut conns = Vec::new();
        conns.extend(self.allies.agent_conns.clone());
        conns.extend(self.opponents.agent_conns.clone());
        conns
    }

    pub fn rcss_conn(&self) -> ServerConfig {
        self.config.server.clone()
    }
}
