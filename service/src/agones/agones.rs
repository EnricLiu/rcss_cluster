use std::fmt::Display;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use log::{debug, info, warn};
use tokio::sync::{mpsc, watch, RwLock};
use chrono::{DateTime, Utc};
use agones::Sdk as AgonesSdk;
use tokio_util::sync::CancellationToken;
use crate::{Error, Result, ServerStatus};
use crate::agones::config::AgonesAutoShutdownConfig;
use super::{AgonesConfig, AgonesArgs, BaseService};
use super::match_composer::MatchComposerClient;

#[derive(Debug, Default)]
pub struct RuntimeCounters {
    pub health_ping_sent: AtomicU64,
    pub health_ping_skipped: AtomicU64,
    pub mc_poll_success: AtomicU64,
    pub mc_poll_failure: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct McLastPoll {
    pub in_match: bool,
    pub error: Option<String>,
    pub polled_at: DateTime<Utc>,
}

pub struct AgonesService {
    sdk:    Arc<RwLock<AgonesSdk>>,
    cfg:    AgonesConfig,
    service: BaseService,
    mc_client: Option<MatchComposerClient>,

    cancel_token: CancellationToken,

    shutdown_tx: watch::Sender<Option<()>>,
    shutdown_rx: watch::Receiver<Option<()>>,

    pub counters: Arc<RuntimeCounters>,
    pub mc_last_poll: Arc<RwLock<Option<McLastPoll>>>,
}

impl std::ops::Deref for AgonesService {
    type Target = BaseService;

    fn deref(&self) -> &Self::Target {
        &self.service
    }
}

impl std::ops::DerefMut for AgonesService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.service
    }
}

impl Display for AgonesService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AgonesService {{ {:?} }}", self.service)
    }
}

impl AgonesService {
    pub async fn from_args(args: AgonesArgs) -> Result<Self> {
        let sdk = agones::Sdk::new(
            args.agones_port,
            args.agones_keep_alive.map(Duration::from_secs),
        ).await.map_err(Error::AgonesSdkFailToConnect)?;

        let base = BaseService::from_args(args.base_args).await;

        let mc_config = args.mc_args.into_config();
        let mc_client = mc_config.as_ref()
            .map(|cfg| MatchComposerClient::new(cfg.client_cfg.clone()));
        

        let config = {
            let mut cfg = AgonesConfig::default();
            cfg.health_check_interval = Duration::from_secs(args.health_check_interval);
            cfg.shutdown.on_finish = args.auto_shutdown_on_finish;
            cfg.sdk.port = args.agones_port;
            cfg.sdk.keep_alive = args.agones_keep_alive.map(Duration::from_secs);
            cfg.match_composer = mc_config;

            cfg
        };

        Ok(AgonesService::new(sdk, base, config, mc_client))
    }

    pub(super) fn new(sdk: agones::Sdk, service: BaseService, config: AgonesConfig, mc_client: Option<MatchComposerClient>) -> Self {
        let sdk = Arc::new(RwLock::new(sdk));
        let cancel_token = CancellationToken::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(None);

        Self { sdk, service, cfg: config, mc_client, cancel_token, shutdown_tx, shutdown_rx,
            counters: Arc::new(RuntimeCounters::default()),
            mc_last_poll: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn spawn(&self) -> Result<()> {
        let mut sdk_guard = self.sdk.write().await;

        self.service.spawn(false).await?;

        let status_rx = self.service.status();
        let health_tx = sdk_guard.health_check();
        let _health_task = tokio::spawn(
            Self::run_health_check(
                status_rx, health_tx,
                self.health_check_interval(),
                self.cancel_token.clone(),
                self.counters.clone(),
            )
        );

        let _shutdown_sig_task = tokio::spawn(
            Self::run_shutdown_signal(
                self.cfg.shutdown.clone(),
                self.service.status(),
                self.shutdown_tx.clone(),
            )
        );

        // Start match_composer players (with retry for sidecar startup race)
        if let Some(mc) = &self.mc_client {
            if let Err(e) = mc.start().await {
                warn!("[AgonesService] MatchComposer start failed, rolling back rcssserver: {e}");
                let _ = self.service.shutdown().await;
                return Err(Error::MatchComposerStartFailed(e));
            }
            info!("[AgonesService] MatchComposer started successfully");
        }

        // Start match_composer status polling task
        if let Some(mc) = &self.mc_client {
            let poll_interval = self.cfg.match_composer
                .as_ref()
                .map(|c| c.status_poll_interval)
                .unwrap_or(Duration::from_secs(5));
            tokio::spawn(Self::run_mc_status_polling(
                mc.clone(),
                self.cancel_token.clone(),
                poll_interval,
                self.counters.clone(),
                self.mc_last_poll.clone(),
            ));
        }

        sdk_guard.ready().await
            .map_err(Error::AgonesSdkReadyFailed)?;

        Ok(())
        // >- sdk WRITE free -<
    }

    async fn run_health_check(
        status_rx: watch::Receiver<ServerStatus>,
        health_tx: mpsc::Sender<()>,
        duration: Duration,
        cancel_token: CancellationToken,
        counters: Arc<RuntimeCounters>,
    ) -> () {
        let mut ticker = tokio::time::interval(duration);

        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("[AgonesService] 'run_health_check': Cancellation token triggered, stopping health checks.");
            }
            _ = async {
                loop {
                    ticker.tick().await;

                    {
                        let status = status_rx.borrow();
                        if !status.is_healthy() {
                            counters.health_ping_skipped.fetch_add(1, Ordering::Relaxed);
                            info!("[AgonesService] Skipping health ping: Server unhealthy [{status:?}]");
                            continue;
                        }
                    }

                    debug!("[AgonesService] Sending health ping to Agones SDK");
                    if health_tx.send(()).await.is_err() {
                        warn!("[AgonesService] Health check task ending: Health channel closed");
                        break;
                    }
                    counters.health_ping_sent.fetch_add(1, Ordering::Relaxed);
                }
            } => {}
        }
    }

    // resolves when service needs to shut down
    pub fn shutdown_signal(&self) -> impl Future<Output = ()> + 'static {
        let mut rx = self.shutdown_rx.clone();
        async move {
            rx.wait_for(|sig| sig.is_some()).await.ok();
            info!("[AgonesService] Shutdown signal received: Cancellation token triggered.");
        }
    }

    async fn run_shutdown_signal(
        shutdown_config: AgonesAutoShutdownConfig,
        status_rx: watch::Receiver<ServerStatus>,
        signal_tx: watch::Sender<Option<()>>,
    ) {
        let cfg = shutdown_config;

        let mut signals: Vec<Pin<Box<dyn Future<Output=()>+Send>>> = vec![];

        if cfg.on_finish {
            signals.push(Box::pin(Self::shutdown_on_finish(status_rx.clone())));
        }

        if signals.is_empty() {
            info!("[AgonesService] 'run_shutdown_signal': No auto-shutdown conditions configured; task exiting.");
            return;
        }

        tokio::select! {
            _ = futures::future::select_all(signals) => {
                info!("[AgonesService] 'run_shutdown_signal': One of the shutdown conditions met.");
                info!("[AgonesService] 'run_shutdown_signal': Triggering cancellation token to shutdown.");
            },
        }

        let _ = signal_tx.send(Some(()));
    }

    async fn shutdown_on_finish(mut status_rx: watch::Receiver<ServerStatus>) {
        match status_rx.wait_for(|s| s.is_finished()).await {
            Ok(_) => info!("[AgonesService] 'shutdown_on_finish': Server status is Finished."),
            Err(_) => warn!("[AgonesService] 'shutdown_on_finish': Status channel closed."),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.cancel_token.cancel();

        // Stop match_composer players first
        if let Some(mc) = &self.mc_client {
            if let Err(e) = mc.stop().await {
                warn!("[AgonesService] Failed to stop match_composer: {e}");
            } else {
                info!("[AgonesService] MatchComposer stopped successfully");
            }
        }

        self.service.shutdown().await?;
        self.sdk.write().await.shutdown().await
            .map_err(Error::AgonesSdkShutdownFailed)?;
        Ok(())
    }

    async fn run_mc_status_polling(
        client: MatchComposerClient,
        cancel_token: CancellationToken,
        interval: Duration,
        counters: Arc<RuntimeCounters>,
        mc_last_poll: Arc<RwLock<Option<McLastPoll>>>,
    ) {
        let mut ticker = tokio::time::interval(interval);
        info!("[AgonesService] MatchComposer status polling started (interval: {}ms)", interval.as_millis());

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("[AgonesService] MatchComposer status polling stopped: cancelled");
                    break;
                }
                _ = ticker.tick() => {
                    let now = Utc::now();
                    match client.status().await {
                        Ok(status) => {
                            counters.mc_poll_success.fetch_add(1, Ordering::Relaxed);
                            *mc_last_poll.write().await = Some(McLastPoll {
                                in_match: status.in_match,
                                polled_at: now,
                                error: None,
                            });
                            debug!("[AgonesService] MatchComposer status: in_match={}", status.in_match);
                        }
                        Err(e) => {
                            counters.mc_poll_failure.fetch_add(1, Ordering::Relaxed);
                            let err_msg = e.to_string();
                            *mc_last_poll.write().await = Some(McLastPoll {
                                in_match: false,
                                polled_at: now,
                                error: Some(err_msg.clone()),
                            });
                            warn!("[AgonesService] MatchComposer status poll failed: {err_msg}");
                        }
                    }
                }
            }
        }
    }

    fn health_check_interval(&self) -> Duration {
        self.cfg.health_check_interval
    }

    pub fn agones_config(&self) -> &AgonesConfig {
        &self.cfg
    }

    /// Whether the cancellation token has been triggered (entering shutdown path).
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Whether the auto-shutdown signal has fired (e.g. game finished).
    pub fn is_shutdown_signalled(&self) -> bool {
        self.shutdown_rx.borrow().is_some()
    }
}