use std::time::Duration;
use super::client::MatchComposerClientConfig;

#[derive(Clone, Debug)]
pub struct MatchComposerConfig {
    pub port: u16,
    pub status_poll_interval: Duration,
    pub client_cfg: MatchComposerClientConfig,
}

impl Default for MatchComposerConfig {
    fn default() -> Self {
        Self {
            port: 6657,
            status_poll_interval: Duration::from_secs(5),
            client_cfg: MatchComposerClientConfig::default(),
        }
    }
}
