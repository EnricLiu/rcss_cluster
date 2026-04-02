use std::time::Duration;
use super::match_composer::MatchComposerConfig;

#[derive(Clone, Debug)]
pub struct AgonesConfig {
    pub health_check_interval: Duration,
    pub sdk: AgonesSdkConfig,
    pub shutdown: AgonesAutoShutdownConfig,
    pub match_composer: Option<MatchComposerConfig>,
}

impl AgonesConfig { pub fn new() -> Self { Self::default() } }
impl Default for AgonesConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(5),
            sdk: AgonesSdkConfig::default(),
            shutdown: AgonesAutoShutdownConfig::default(),
            match_composer: None,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct AgonesSdkConfig {
    pub port: Option<u16>,
    pub keep_alive: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct AgonesAutoShutdownConfig {
    pub on_finish: bool,
}

impl Default for AgonesAutoShutdownConfig {
    fn default() -> Self {
        Self { on_finish: true }
    }
}