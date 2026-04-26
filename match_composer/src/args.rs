use std::net::IpAddr;
use std::path::PathBuf;
use clap::Parser;
use common::utils::logging::LoggingArgs;

#[derive(Debug, Clone, Parser)]
#[command(name = "match_composer", about = "Match Composer HTTP server")]
pub struct Args {
    /// HTTP server listen address
    #[arg(long, env = "MC_HOST", default_value = "0.0.0.0", help = "Server IP to bind")]
    pub host: IpAddr,

    /// HTTP server listen port
    #[arg(long, env = "MC_PORT", default_value = "7777", help = "Server port to bind")]
    pub port: u16,
    
    #[arg(long, env = "MC_PLAYER_SPAWN_DELAY", default_value = "500", help = "Delay in milliseconds between spawning individual players")]
    pub player_spawn_delay: u64,
    #[arg(long, env = "MC_TEAM_SPAWN_DELAY", default_value = "1000", help = "Delay in milliseconds between spawning teams")]
    pub team_spawn_delay: u64,
    #[arg(long, env = "MC_TEAM_SPAWN_CONCURRENT_EN", default_value = "false", help = "Whether to spawn teams concurrently")]
    pub team_spawn_concurrent_en: bool,
    
    /// RCSS server host for agent communication
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1", help = "RCSS wrapped server host for player communication")]
    pub rcss_host: IpAddr,
    
    /// RCSS server port for agent communication
    #[arg(long, env = "SERVER_PORT", default_value = "6666", help = "RCSS wrapped server port for player communication")]
    pub rcss_port: u16,

    /// RCSS server UDP port for player communication
    #[arg(long, env = "SERVER_UDP_PORT_PLAYER", default_value = "6657", help = "RCSS server UDP port for player communication")]
    pub rcss_player_port: u16,
    
    /// RCSS server UDP port for trainer communication
    #[arg(long, env = "SERVER_UDP_PORT_TRAINER", default_value = "6658", help = "RCSS server UDP port for trainer communication")]
    pub rcss_trainer_port: u16,

    /// RCSS server UDP port for coach communication
    #[arg(long, env = "SERVER_UDP_PORT_COACH", default_value = "6659", help = "RCSS server UDP port for coach communication")]
    pub rcss_coach_port: u16,
    
    /// Path to the hub/registry directory containing agent images
    #[arg(long, env = "MC_HUB_PATH", default_value = "./hub", help = "Path to the hub/registry directory containing agent images")]
    pub hub_path: PathBuf,

    /// Root directory for match logs
    #[arg(long, env = "MC_PLAYER_LOG_ROOT", default_value = "./players", help = "Root directory for player logs")]
    pub player_log_root_dir: Option<PathBuf>,


    #[arg(short='f', long, env = "MC_CONFIG_FILE", help = "Path to the ConfigV1 JSON file, exclusive with -a or --agones")]
    pub file: Option<PathBuf>,

    #[arg(short='a', long, env = "MC_AGONES_EN", default_value = "false", help = "Enable Agones SDK integration for GameServer lifecycle management")]
    pub agones: bool,

    #[arg(long, env = "AGONES_GRPC_PORT", default_value = "9357", help = "Port for Agones SDK gRPC communication")]
    pub agones_grpc_port: Option<u16>,

    #[arg(long, env = "AGONES_KEEP_ALIVE_S", default_value = "30", help = "Interval in seconds for sending keep-alive messages to Agones")]
    pub agones_keep_alive: Option<u64>,

    #[command(flatten)]
    pub log_args: LoggingArgs,

    #[arg(long, env = "MC_STDIO_LOG_PATH", help = "Path to the log file for standard output and error of the match composer, if not set, logs will be printed to console")]
    pub stdio_log_path: Option<PathBuf>,
}
