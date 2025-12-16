use std::fmt::Display;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use log::{debug, info, warn};
use tokio::sync::{mpsc, watch, RwLock};
use tokio_util::sync::CancellationToken;
use agones::Sdk as AgonesSdk;
use process::CoachedProcessSpawner;

use crate::{Error, Result, ServerStatus};
use super::BaseService;

#[derive(Debug, Clone)]
pub struct Config {
    pub health_check_interval: Duration,
    pub auto_shutdown: AutoShutdownConfig,
}

#[derive(Debug, Clone)]
pub struct AutoShutdownConfig {
    pub on_finish: bool,
}

pub struct AgonesService {
    sdk:    Arc<RwLock<AgonesSdk>>,
    cfg:    Config,
    service: BaseService,

    cancel_token: CancellationToken,

    shutdown_tx: watch::Sender<Option<()>>,
    shutdown_rx: watch::Receiver<Option<()>>,
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
    pub async fn new(spawner: CoachedProcessSpawner) -> Self {
        let sdk = AgonesSdk::new(None, None).await.unwrap();
        let sdk = Arc::new(RwLock::new(sdk));

        let service = BaseService::new(spawner).await;
        let config = Config {
            health_check_interval: Duration::from_secs(5),
            auto_shutdown: AutoShutdownConfig {
                on_finish: true,
            },
        };

        let cancel_token = CancellationToken::new();
        let (shutdown_tx, shutdown_rx) = watch::channel(None);

        Self { sdk, service, cfg: config, cancel_token, shutdown_tx, shutdown_rx }
    }

    pub async fn spawn(&self) -> Result<()> {
        // >- sdk WRITE lock -<
        let mut sdk_guard = self.sdk.write().await;

        self.service.spawn(false).await?;

        let status_rx = self.service.status();
        let health_tx = sdk_guard.health_check();
        let _health_task = tokio::spawn(
            Self::run_health_check(
                status_rx, health_tx,
                self.health_check_interval(),
                self.cancel_token.clone()
            )
        );

        let _shutdown_sig_task = tokio::spawn(
            Self::run_shutdown_signal(
                self.cfg.auto_shutdown.clone(),
                self.service.status(),
                self.shutdown_tx.clone(),
            )
        );

        sdk_guard.ready().await.expect("TODO: panic message");

        Ok(())
        // >- sdk WRITE free -<
    }

    async fn run_health_check(
        status_rx: watch::Receiver<ServerStatus>,
        health_tx: mpsc::Sender<()>,
        duration: Duration,
        cancel_token: CancellationToken,
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
                            debug!("[AgonesService] Skipping health ping: Server unhealthy [{status:?}]");
                            continue;
                        }
                    }

                    debug!("[AgonesService] Sending health ping to Agones SDK");
                    if health_tx.send(()).await.is_err() {
                        warn!("[AgonesService] Health check task ending: Health channel closed");
                        break;
                    }
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
        shutdown_config: AutoShutdownConfig,
        status_rx: watch::Receiver<ServerStatus>,
        signal_tx: watch::Sender<Option<()>>,
    ) {
        let cfg = shutdown_config;

        let mut signals: Vec<Pin<Box<dyn Future<Output=()>+Send>>> = vec![];

        if cfg.on_finish {
            signals.push(Box::pin(Self::shutdown_on_finish(status_rx.clone())));
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

    pub async fn shutdown(&mut self) -> Result<()> {
        self.cancel_token.cancel();
        self.service.shutdown().await?;
        self.sdk.write().await.shutdown().await.expect("TODO: panic message");
        Ok(())
    }

    fn health_check_interval(&self) -> Duration {
        self.cfg.health_check_interval
    }
}