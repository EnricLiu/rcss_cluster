use std::net::Ipv4Addr;
use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "match_composer", about = "Match Composer HTTP server")]
pub struct Args {
    /// HTTP server listen address
    #[arg(long, env = "HOST", default_value = "0.0.0.0")]
    pub host: Ipv4Addr,

    /// HTTP server listen port
    #[arg(long, env = "PORT", default_value = "6657")]
    pub port: u16,
    
    /// RCSS server host for agent communication
    #[arg(long, env = "RCSS_HOST", default_value = "127.0.0.1")]
    pub rcss_host: Ipv4Addr,
    
    /// RCSS server port for agent communication
    #[arg(long, env = "RCSS_PORT", default_value = "55555")]
    pub rcss_port: u16,

    /// Path to the hub/registry directory containing agent images
    #[arg(long, env = "HUB_PATH", default_value = "sidecars/match_composer/hub")]
    pub hub_path: PathBuf,

    /// Root directory for match logs
    #[arg(long, env = "LOG_ROOT", default_value = None)]
    pub log_root: Option<PathBuf>,


    #[arg(short='f', long, env = "CONFIG_FILE", help = "Path to the ConfigV1 JSON file, exclusive with -a or --agones")]
    pub file: Option<PathBuf>,

    #[arg(short='a', long, env = "AGONES_ENABLED", default_value = "false", help = "Enable Agones SDK integration for GameServer lifecycle management")]
    pub agones: bool,

    #[arg(long, env = "AGONES_GRPC_PORT", default_value = "9357", help = "Port for Agones SDK gRPC communication")]
    pub agones_grpc_port: Option<u16>,

    #[arg(long, env = "AGONES_KEEP_ALIVE", default_value = "30", help = "Interval in seconds for sending keep-alive messages to Agones")]
    pub agones_keep_alive: Option<u64>
}
