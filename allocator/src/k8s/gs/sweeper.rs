use std::time::Duration;

use log::{debug, info, warn};

use super::K8sClient;
use super::crd::GameServer;
use super::lifecycle::{CREATED_AT_ANNOTATION, LAST_HEARTBEAT_ANNOTATION, now_unix_secs};

#[derive(Debug, Clone)]
pub struct GsSweepConfig {
    pub interval: Duration,
    pub ready_idle_ttl: Option<Duration>,
    pub lease_ttl: Option<Duration>,
    pub hard_ttl: Option<Duration>,
}

impl GsSweepConfig {
    pub fn enabled(&self) -> bool {
        !self.interval.is_zero()
    }
}

impl K8sClient {
    pub async fn run_managed_gs_sweeper(self, config: GsSweepConfig) {
        if !config.enabled() {
            info!("Managed direct GameServer sweeper disabled");
            return;
        }

        info!("Managed direct GameServer sweeper started: {config:?}");
        let mut interval = tokio::time::interval(config.interval);
        loop {
            interval.tick().await;
            if let Err(err) = self.sweep_managed_gs(&config).await {
                warn!("Managed direct GameServer sweep failed: {err:?}");
            }
        }
    }

    pub async fn sweep_managed_gs(&self, config: &GsSweepConfig) -> super::Result<usize> {
        let now = now_unix_secs();
        let mut deleted = 0usize;

        for gs in self.list_managed_direct_gs().await? {
            let name = gs.name().to_string();
            let Some(reason) = should_delete(&gs, config, now) else {
                continue;
            };

            info!("Deleting managed direct GameServer[{name}]: {reason}");
            match self.drop_gs(&name).await {
                Ok(()) => deleted += 1,
                Err(err) => warn!("Failed to delete managed direct GameServer[{name}]: {err:?}"),
            }
        }

        if deleted > 0 {
            info!("Managed direct GameServer sweep deleted {deleted} GameServers");
        } else {
            debug!("Managed direct GameServer sweep deleted no GameServers");
        }

        Ok(deleted)
    }
}

fn should_delete(gs: &GameServer, config: &GsSweepConfig, now: u64) -> Option<String> {
    let created_at = annotation_u64(gs, CREATED_AT_ANNOTATION).or_else(|| {
        gs.metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| u64::try_from(ts.0.as_second()).ok())
    })?;
    let age = now.saturating_sub(created_at);

    if ttl_expired(config.hard_ttl, age) {
        return Some(format!("hard ttl expired age={age}s"));
    }

    let state = gs
        .status
        .as_ref()
        .map(|s| s.state.as_str())
        .unwrap_or("Unknown");
    match state {
        "Ready" => {
            if ttl_expired(config.ready_idle_ttl, age) {
                Some(format!("ready idle ttl expired age={age}s"))
            } else {
                None
            }
        }
        "Allocated" => {
            let last_heartbeat =
                annotation_u64(gs, LAST_HEARTBEAT_ANNOTATION).unwrap_or(created_at);
            let idle = now.saturating_sub(last_heartbeat);
            if ttl_expired(config.lease_ttl, idle) {
                Some(format!("lease ttl expired idle={idle}s age={age}s"))
            } else {
                None
            }
        }
        "Shutdown" | "Error" | "Unhealthy" => {
            if ttl_expired(config.ready_idle_ttl, age) {
                Some(format!("terminal/unhealthy state={state} age={age}s"))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn annotation_u64(gs: &GameServer, key: &str) -> Option<u64> {
    gs.metadata
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.get(key))
        .and_then(|value| value.parse::<u64>().ok())
}

fn ttl_expired(ttl: Option<Duration>, elapsed_secs: u64) -> bool {
    ttl.is_some_and(|ttl| !ttl.is_zero() && elapsed_secs >= ttl.as_secs())
}
