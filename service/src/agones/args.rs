use clap::Parser;
use super::BaseArgs;
use super::match_composer::MatchComposerArgs;

#[derive(Parser, Debug)]
pub struct AgonesArgs {
    #[clap(long, env = "AGONES_GRPC_PORT", default_value = "9357", help = "Agones SDK port, default at 9357")]
    pub agones_port: Option<u16>,
    #[clap(long, env = "AGONES_KEEP_ALIVE_S", default_value = "30", help = "Agones SDK keep alive duration in seconds, default 30s")]
    pub agones_keep_alive: Option<u64>,

    #[clap(long, env = "AGONES_HEALTH_INTERVAL_S", default_value_t = 5, help = "Agones health check interval in seconds")]
    pub health_check_interval: u64,
    #[clap(long, env = "CONTAINER_AUTO_SHUTDOWN_ON_FINISH_EN", default_value_t = true, help = "Auto shutdown the server when the match is finished")]
    pub auto_shutdown_on_finish: bool,

    #[clap(flatten)]
    pub base_args: BaseArgs,

    #[clap(flatten)]
    pub mc_args: MatchComposerArgs,
}
