mod error;
mod response;
mod routes;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use chrono::{Duration, Utc};
use log::{debug, error, info};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex, RwLock};
use common::types::Side;
pub use error::Error;
pub use response::StatusResponse;

use crate::composer::MatchComposer;
use crate::config::MatchComposerConfig;
use crate::schema::v1::ConfigV1;

enum ComposerState {
    Idle,
    Running {
        composer: MatchComposer,
        started_at: chrono::DateTime<Utc>,
    },
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<ComposerState>>,
    cfg: Arc<RwLock<Option<ConfigV1>>>,
    hub_path: Arc<PathBuf>,
    log_root: Arc<PathBuf>,
}

impl AppState {
    const CLEANER_POLL_INTERVAL: Duration = Duration::seconds(1);
    const CLEANER_TIMEOUT: Duration = Duration::seconds(30);

    pub fn new(
        hub_path: PathBuf,
        log_root: PathBuf,
        shutdown_notifier: Option<oneshot::Receiver<()>>,
    ) -> Self {
        let inner = Arc::new(Mutex::new(ComposerState::Idle));

        if let Some(notifier) = shutdown_notifier {
            tokio::spawn(Self::run_shutdown_cleaner(inner.clone(), notifier));
        }

        Self {
            inner,
            cfg: Arc::new(RwLock::new(None)),
            hub_path: Arc::new(hub_path),
            log_root: Arc::new(log_root),
        }
    }

    async fn run_shutdown_cleaner(
        inner: Arc<Mutex<ComposerState>>,
        notifier: oneshot::Receiver<()>,
    ) {
        notifier.await.ok();
        debug!("[AppState] Shutdown notifier received, cleaning up composer...");

        let mut interval =
            tokio::time::interval(Self::CLEANER_POLL_INTERVAL.to_std()
                .expect("CLEANER_POLL_INTERVAL must be a positive duration"));
        let start_at = Utc::now();

        loop {
            interval.tick().await;

            if let Ok(mut guard) = inner.try_lock() {
                if let ComposerState::Running { composer, .. } = &mut *guard {
                    composer.shutdown().await;
                    info!(
                        "[AppState] Composer shutdown completed in {}ms.",
                        (Utc::now() - start_at).num_milliseconds()
                    );
                } else {
                    debug!("[AppState] Composer was already idle at shutdown.");
                }
                *guard = ComposerState::Idle;
                break;
            }

            if (Utc::now() - start_at) > Self::CLEANER_TIMEOUT {
                error!(
                    "[AppState] Composer shutdown timed out after {} seconds.",
                    Self::CLEANER_TIMEOUT.num_seconds()
                );
                break;
            }
        }
    }

    /// Start composer from a `ConfigV1`. Returns error if already running.
    pub async fn start(&self, config: Option<ConfigV1>) -> Result<(), Error> {
        let config = self.build_config(config).await?;

        let mut guard = self.inner.lock().await;
        if matches!(*guard, ComposerState::Running { .. }) {
            return Err(Error::conflict(
                "Composer is already running. Call /stop or /restart first.",
            ));
        }

        self.do_start(config, &mut guard).await
    }

    /// Stop the running composer. Returns error if already idle.
    pub async fn stop(&self) -> Result<(), Error> {
        let mut guard = self.inner.lock().await;
        match &mut *guard {
            ComposerState::Idle => Err(Error::bad_request("Composer is not running.")),
            ComposerState::Running { composer, .. } => {
                composer.shutdown().await;
                *guard = ComposerState::Idle;
                Ok(())
            }
        }
    }

    /// Stop (if running) then start with new config.
    pub async fn restart(&self, config: Option<ConfigV1>) -> Result<(), Error> {
        let config = self.build_config(config).await?;

        let mut guard = self.inner.lock().await;
        if let ComposerState::Running { composer, .. } = &mut *guard {
            composer.shutdown().await;
        }
        *guard = ComposerState::Idle;
        self.do_start(config, &mut guard).await
    }

    /// Return current state and agent connections.
    pub async fn status(&self) -> Json<StatusResponse> {
        let guard = self.inner.lock().await;
        match &*guard {
            ComposerState::Idle => Json(
                StatusResponse {
                    state: "idle",
                    team_l: None,
                    team_r: None,
                    started_at: None
                }
            ),
            ComposerState::Running { composer, started_at } => {
                let left = composer.team(Side::LEFT);
                let right = composer.team(Side::RIGHT);
                Json(
                    StatusResponse {
                        state: "running",
                        team_l: Some(left),
                        team_r: Some(right),
                        started_at: Some(*started_at),
                    }
                )
            }
        }
    }

    async fn build_config(&self, config: Option<ConfigV1>) -> Result<ConfigV1, Error> {
        match config {
            Some(cfg) => Ok(cfg),
            _ => {
                self.cfg.read().await.clone().ok_or_else(
                    || Error::bad_request("No config provided and no previous config found.")
                )
            }
        }
    }

    async fn do_start(
        &self,
        config: ConfigV1,
        state: &mut ComposerState,
    ) -> Result<(), Error> {
        let started_at = Utc::now();
        let log_root = self.log_root.join(started_at.format("%Y%m%d_%H%M%S").to_string());

        let composer_config =
            MatchComposerConfig::from_schema(config.clone(), Some(log_root))
                .map_err(|e| Error::bad_request(format!("Invalid config: {e}")))?;

        let mut composer = MatchComposer::new(composer_config, self.hub_path.as_ref())
            .map_err(|e| Error::internal(format!("Failed to create composer: {e}")))?;

        composer
            .spawn_players()
            .await
            .map_err(|e| Error::internal(format!("Failed to spawn players: {e}")))?;

        *state = ComposerState::Running { composer, started_at };
        *self.cfg.write().await = Some(config);
        Ok(())
    }
}

fn router(state: AppState) -> Router {
    routes::route().with_state(state)
}

pub async fn listen(addr: SocketAddr, config: ConfigV1, hub_path: PathBuf, log_root: PathBuf) {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let state = AppState::new(hub_path, log_root, Some(shutdown_rx));
    
    let _state = state.clone();
    tokio::spawn(async move {
        _state.start(Some(config)).await
    });
    
    let app = router(state);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    info!("match_composer listening on {addr}");

    let signal = async move {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => { debug!("[Server] Ctrl-C received, shutting down..."); },
            _ = terminate => { debug!("[Server] SIGTERM received, shutting down..."); },
        }

        let _ = shutdown_tx.send(());
        debug!("[Server] Shutdown signal sent to AppState cleaner.");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(signal)
        .await
        .expect("Server error");

    info!("Server shut down.");
}
