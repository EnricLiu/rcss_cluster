mod error;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::Router;
use chrono::{Duration, Utc};
use log::{debug, error, info};
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use common::types::Side;
pub use error::{Error, Result};
use crate::agones::AgonesMetaData;
use crate::composer::{Match, MatchComposer, MatchComposerConfig};
use crate::info::{GameInfo, TeamInfo};
use crate::team::{TeamStatus};

enum ComposerState {
    Idle,
    Running {
        composer: MatchComposer,
        started_at: chrono::DateTime<Utc>,
    },
}

impl ComposerState {
    pub fn kind(&self) -> &'static str {
        match self {
            ComposerState::Idle => "idle",
            ComposerState::Running { .. } => "running",
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    composer: Arc<MatchComposer>,
    game: Arc<RwLock<Option<Match>>>,
}

impl AppState {
    const CLEANER_POLL_INTERVAL: Duration = Duration::seconds(1);
    const CLEANER_TIMEOUT: Duration = Duration::seconds(30);

    pub fn new(
        composer_config: MatchComposerConfig,
        shutdown_notifier: Option<oneshot::Receiver<()>>,
    ) -> Result<Self> {
        let composer = MatchComposer::new(composer_config)
            .map_err(|e| Error::internal(e.to_string()))?;

        let composer = Arc::new(composer);
        let game = Arc::new(RwLock::new(None));
        if let Some(notifier) = shutdown_notifier {
            tokio::spawn(Self::run_shutdown_cleaner(game.clone(), notifier));
        }

        Ok(
            Self {
                composer,
                game,
            }
        )
    }

    async fn run_shutdown_cleaner(
        game: Arc<RwLock<Option<Match>>>,
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

            if  let Ok(mut guard) = game.try_write() &&
                let Some(game) = guard.as_mut() {
                    if let Err(e) = game.shutdown().await {
                        error!("[AppState] Error during composer shutdown: {e}");
                    }
                    info!(
                        "[AppState] Composer shutdown completed in {}ms.",
                        (Utc::now() - start_at).num_milliseconds()
                    );
                    *guard = None;
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
    pub async fn start(&self, meta: Option<AgonesMetaData>) -> Result<()> {
        let mut guard = self.game.write().await;
        
        if guard.is_some() {
            return Err(Error::conflict(
                "Composer is already running. Call /stop or /restart first.",
            ));
        }
        
        self.update_meta(meta).await;
        self.do_start(&mut guard).await
    }

    /// Stop the running composer. Returns error if already idle.
    pub async fn stop(&self) -> Result<()> {
        let mut guard = self.game.write().await;
        if let Some(game) = guard.as_mut() {
            if let Err(e) = game.shutdown().await {
                error!("[AppState] Error during composer shutdown: {e}");
                return Err(Error::internal(format!("Failed to shutdown running composer: {e}")));
            }
            *guard = None;
            Ok(())
        } else {
            Err(Error::bad_request("Composer is not running."))
        }
    }

    /// Stop (if running) then start with new meta.
    pub async fn restart(&self, meta: Option<AgonesMetaData>) -> Result<()> {
        let mut guard = self.game.write().await;

        if let Some(game) = guard.as_mut() {
            if let Err(e) = game.shutdown().await {
                error!("[AppState] Error during composer shutdown: {e}");
                return Err(Error::internal(format!("Failed to shutdown running composer: {e}")));
            }
            *guard = None;
        }
        
        self.update_meta(meta).await;
        self.do_start(&mut guard).await
    }

    pub async fn team_status(&self, side: Side) -> Option<TeamStatus> {
        self.game.read().await
            .as_ref()
            .map(|game| {
                game.team(side).status_now()
            })
    }
    
    pub async fn team_info(&self, side: Side) -> Option<TeamInfo> {
        self.game.read().await
            .as_ref()
            .map(|game| {
                game.team(side).info()
            })
    }
    
    pub async fn match_info(&self) -> Option<GameInfo> {
        self.game.read().await
            .as_ref()
            .map(|game| game.info())
    }

    async fn do_start(
        &self,
        game: &mut Option<Match>,
    ) -> Result<()> {
        let started_at = Utc::now();
        let log_root = started_at.format("%Y%m%d_%H%M%S").to_string();
        
        let new_game = self.composer
            .make_match(log_root).await
            .map_err(|e| Error::internal(format!("Failed to spawn players: {e}")))?;

        *game = Some(new_game);
        Ok(())
    }
    
    async fn update_meta(&self, meta: Option<AgonesMetaData>) {
        if let Some(meta) = meta {
            info!("[AppState] Composer meta updating, meta: {meta:?}");
            self.composer.set_meta(meta).await;
        }
    }
}

fn router(state: AppState) -> Router {
    routes::route("/", state)
}

pub async fn listen(addr: SocketAddr, meta: AgonesMetaData, config: MatchComposerConfig) -> Result<JoinHandle<()>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let state = AppState::new(config, Some(shutdown_rx))?;
    
    state.update_meta(Some(meta)).await;
    // let _state = state.clone();
    // tokio::spawn(async move {
    //     _state.start(Some(meta)).await
    // });
    
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
            signal(SignalKind::terminate())
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

    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(signal)
            .await
            .expect("Server error");
    });

    info!("Server shut down.");
    Ok(server)
}
