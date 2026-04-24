use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct BaseArgs {
    #[clap(long, env = "RCSSSERVER_PLAYER_UDP", default_value_t = 6000, help = "RCSS player udp port")]
    pub player_port: u16,
    #[clap(long, env = "RCSSSERVER_TRAINER_UDP", default_value_t = 6001, help = "RCSS trainer udp port")]
    pub trainer_port: u16,
    #[clap(long, env = "RCSSSERVER_COACH_UDP", default_value_t = 6002, help = "RCSS coach udp port")]
    pub coach_port: u16,
    #[clap(long, env = "RCSSSERVER_SYNC_MODE_EN", default_value_t = true, help = "RCSS sync mode")]
    pub rcss_sync: bool,
    #[clap(long, env = "RCSSSERVER_LOG_DIR", default_value = "./games", help = "RCSS log directory")]
    pub rcss_game_log_dir: PathBuf,
    #[clap(long, env = "RCSSSERVER_MAX_TIMESTEP", default_value_t = 6000, help = "Total timesteps")]
    pub rcss_max_timesteps: u16,
    
    #[clap(long, env = "TRAINER_HALF_TIME_AUTO_START_EN", default_value_t = false, help = "Auto start when half-time(3000) is reached")]
    pub half_time_auto_start: bool,
    
    #[clap(long, env = "LOGGER_STDOUT_ALWAYS_EN", default_value_t = true, help = "Always log stdout and stderr")]
    pub always_log_stdout: bool,
    #[clap(long, env = "RCSSSERVER_STDIO_LOG_PATH", default_value = "./rcss.log", help = "RCSSServer wrapped process stdout/stderr log file")]
    pub rcss_stdio_log_path: PathBuf,
}