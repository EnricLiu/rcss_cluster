use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use super::config::MatchComposerConfig;
use super::client::MatchComposerClientConfig;


#[derive(clap::Parser, Debug)]
pub struct MatchComposerArgs {
    #[arg(long, env = "MATCH_COMPOSER_EN", group = "match_composer_group", help = "Enable Match Composer Sidecar Mode")]
    pub match_composer: bool,

    #[arg(long, env = "MATCH_COMPOSER_HOST", default_value = "127.0.0.1", requires = "match_composer_group", help = "Match Composer HTTP server host, default at 127.0.0.1")]
    pub mc_host: IpAddr,

    #[arg(long, env = "MATCH_COMPOSER_PORT", default_value_t = 7777, requires = "match_composer_group", help = "Match Composer HTTP server port, default at 7777")]
    pub mc_port: u16,

    #[arg(long, env = "MATCH_COMPOSER_STATUS_POLL_INTERVAL_MS", default_value_t = 5000, requires = "match_composer_group", help = "Match Composer status poll interval in milliseconds, default at 5000ms")]
    pub mc_status_poll_interval: u64,

    #[arg(long, env = "MATCH_COMPOSER_CLIENT_CONNECT_TIMEOUT_MS", default_value_t = 5000, requires = "match_composer_group", help = "Match Composer client connect timeout, default at 5s")]
    pub mc_client_connect_timeout: u64,
    #[arg(long, env = "MATCH_COMPOSER_CLIENT_REQUEST_TIMEOUT_MS", default_value_t = 60000, requires = "match_composer_group", help = "Match Composer client request timeout, default at 60s")]
    pub mc_client_request_timeout: u64,
    
    #[arg(long, env = "MATCH_COMPOSER_CLIENT_RETRY_BACKOFF_EN", default_value = "false", requires = "match_composer_group", help = "Enable Match Composer client start request retry backoff with jitter, default at false")]
    pub mc_client_retry_backoff_en: Option<String>,
    #[arg(long, env = "MATCH_COMPOSER_CLIENT_MAX_RETRIES", default_value_t = 3, requires = "match_composer_group", help = "Match Composer client start request max retries, default at 3")]
    pub mc_client_max_retries: u32,
    #[arg(long, env = "MATCH_COMPOSER_CLIENT_RETRY_DELAY_MS", default_value_t = 1000, requires = "match_composer_group", help = "Match Composer client start request retry base in milliseconds, default at 1000ms")]
    pub mc_client_retry_base: u64,
}

impl MatchComposerArgs {
    pub fn is_enabled(&self) -> bool {
        self.match_composer
    }

    pub fn into_config(self) -> Option<MatchComposerConfig> {
        if !self.is_enabled() { return None }
        let mc_client = MatchComposerClientConfig {
            addr: SocketAddr::new(self.mc_host, self.mc_port),
            connect_timeout: Duration::from_millis(self.mc_client_connect_timeout),
            request_timeout: Duration::from_millis(self.mc_client_request_timeout),
            retry_base: Duration::from_millis(self.mc_client_retry_base),
            max_retries: self.mc_client_max_retries,
            retry_backoff: self.mc_client_retry_backoff_en.is_some(),
        };

        let config = MatchComposerConfig {
            port: self.mc_port,
            status_poll_interval: Duration::from_millis(self.mc_status_poll_interval),
            client_cfg: mc_client,
        };

        Some(config)
    }
}
