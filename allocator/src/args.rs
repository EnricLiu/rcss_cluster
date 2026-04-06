use std::net::Ipv4Addr;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Scheduling {
    Packed,
    Distributed,
}

impl Scheduling {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scheduling::Packed => "Packed",
            Scheduling::Distributed => "Distributed",
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(name = "allocator")]
#[command(about = "Custom Allocator for Agones GameServer allocation")]
pub struct Args {
    /// HTTP server bind address
    #[arg(long, env = "ALLOCATOR_HOST", default_value = "0.0.0.0", help = "Server IP to bind")]
    pub host: Ipv4Addr,
    
    #[arg(long, env = "ALLOCATOR_HTTP_PORT", default_value_t = 5555, help = "Http Server port to bind")]
    pub http_port: u16,
    
    /// Kubernetes namespace
    #[arg(long, env = "AGONES_FLEET_NAMESPACE", default_value = "rcss-env-dev", help = "Kubernetes namespace, where the Fleet(GameServers) are allocated")]
    pub namespace: String,

    /// Bearer token for authentication (optional)
    #[arg(long, env = "ALLOCATOR_AUTH_TOKEN")]
    pub auth_token: Option<String>,

    /// Scheduling strategy for GameServer allocation
    #[arg(long, env = "AGONES_GSA_SCHEDULE_STRATEGY", default_value = "packed")]
    pub scheduling: Scheduling,
}

