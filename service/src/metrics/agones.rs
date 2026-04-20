use serde::Serialize;
use chrono::{DateTime, Utc};
use std::sync::atomic::Ordering;

#[derive(Serialize, Debug, Clone)]
pub struct McLastPollInfo {
    pub in_match: bool,
    pub polled_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AgonesRuntimeInfo {
    /// `true` once `CancellationToken::cancel()` has been called.
    pub cancelled: bool,
    /// `true` once an auto-shutdown condition (e.g. game finished) has triggered.
    pub shutdown_signalled: bool,

    /// Total health pings successfully sent to the Agones SDK.
    pub health_ping_sent: u64,
    /// Health pings skipped because the server was unhealthy.
    pub health_ping_skipped: u64,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc_poll_success: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc_poll_failure: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc_last_poll: Option<McLastPollInfo>,
}

impl crate::Service {
    pub async fn agones_runtime_info(&self) -> AgonesRuntimeInfo {
        let counters = &self.counters;
        let has_mc = self.agones_config().match_composer.is_some();

        let mc_last_poll = {
            let guard = self.mc_last_poll.read().await;
            guard.as_ref().map(|s| McLastPollInfo {
                in_match: s.in_match,
                polled_at: s.polled_at,
                error: s.error.clone(),
            })
        };

        AgonesRuntimeInfo {
            cancelled: self.is_cancelled(),
            shutdown_signalled: self.is_shutdown_signalled(),
            health_ping_sent: counters.health_ping_sent.load(Ordering::Relaxed),
            health_ping_skipped: counters.health_ping_skipped.load(Ordering::Relaxed),
            mc_poll_success: has_mc.then(|| counters.mc_poll_success.load(Ordering::Relaxed)),
            mc_poll_failure: has_mc.then(|| counters.mc_poll_failure.load(Ordering::Relaxed)),
            mc_last_poll,
        }
    }
}

