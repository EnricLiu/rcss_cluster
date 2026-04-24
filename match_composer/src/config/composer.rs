use std::path::PathBuf;
use std::time::Duration;
use super::RcssServerConfig;

#[derive(Clone, Debug)]
pub struct MatchComposerConfig {
    pub server: RcssServerConfig,
    pub player_log_root: Option<PathBuf>,
    pub registry_path: PathBuf,
    pub player_spawn_delay: Duration,
    pub team_spawn_delay: Duration,
    pub concurrent_team_spawn: bool,
}
