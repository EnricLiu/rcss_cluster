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
    #[arg(long, env = "HOST", default_value = "0.0.0.0", help = "Server IP to bind")]
    pub host: Ipv4Addr,
    
    #[arg(long, env = "PORT_HTTP", default_value_t = 6666, help = "Http Server port to bind")]
    pub http_port: u16,
    
    /// Kubernetes namespace
    #[arg(long, env = "NAMESPACE", default_value = "rcss-agones")]
    pub namespace: String,

    /// Bearer token for authentication (optional)
    #[arg(long, env = "AUTH_TOKEN")]
    pub auth_token: Option<String>,

    /// Scheduling strategy for GameServer allocation
    #[arg(long, env = "SCHEDULING", default_value = "packed")]
    pub scheduling: Scheduling,
}

