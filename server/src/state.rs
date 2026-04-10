use std::sync::Arc;
use std::collections::HashMap;

use log::{debug, error, info, warn};
use uuid::Uuid;
use tokio::sync::{oneshot, watch};
use chrono::{Utc, Duration};
use common::client::Info as ClientInfo;
use service::Service;

use crate::proxy::manager::SessionManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStateStatus {
    Running,
    ShuttingDown,
    Stopped,
}

impl AppStateStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn as_shutting_down(&mut self) {
        *self = Self::ShuttingDown;
    }

    pub fn as_stopped(&mut self) {
        *self = Self::Stopped;
    }
}


#[derive(Clone)]
pub struct AppState {
    pub(crate) service: Arc<Service>,
    pub(crate) session: Arc<SessionManager>,

    pub status_rx: watch::Receiver<AppStateStatus>,
}

impl AppState {
    pub const CLEANER_POLL_INTERVAL: Duration = Duration::seconds(1);
    pub const CLEANER_TIMEOUT: Duration = Duration::seconds(30);
    
    pub fn new(service: Service, shutdown_notifier: Option<oneshot::Receiver<()>>) -> Self {
        let service = Arc::new(service);

        let (status_tx, status_rx) = watch::channel(AppStateStatus::Running);

        if let Some(shutdown_notifier) = shutdown_notifier {
            tokio::spawn(Self::run_wait_for_shutdown_cleaner(
                service.clone(), shutdown_notifier, status_tx));
        }
        
        Self {
            service,
            session: Arc::new(SessionManager::new()),
            status_rx,
        }
    }
    
    async fn run_wait_for_shutdown_cleaner(
        mut service: Arc<Service>,
        shutdown_notifier: oneshot::Receiver<()>,
        mut status_tx: watch::Sender<AppStateStatus>,
    ) {
        shutdown_notifier.await.ok();
        debug!("[AppState] Shutdown notifier received, starting polling service shutdown...");

        status_tx.send(AppStateStatus::ShuttingDown).ok();

        let mut interval = tokio::time::interval(Self::CLEANER_POLL_INTERVAL.to_std().unwrap());
        let start_at = Utc::now();

        let success = loop {
            interval.tick().await;

            if let Some(service) = Arc::get_mut(&mut service) {
                match service.shutdown().await {
                    Ok(_) => {
                        info!("[AppState] Service shutdown completed in {}ms.", 
                            (Utc::now() - start_at).num_milliseconds());
                        break true;
                    },
                    Err(e) => {
                        error!("[AppState] Service shutdown failed: {}. Retrying...", e);
                    },
                }
            }

            if (Utc::now() - start_at) > Self::CLEANER_TIMEOUT {
                warn!("[AppState] Service shutdown timed out after {} seconds.",
                    Self::CLEANER_TIMEOUT.num_seconds());
                break false;
            }
        };

        if !success {
            warn!("[AppState] Force shutting down regardless of service state.");
            // shutdown no waiting anymore
            if let Err(e) = service.shutdown().await {
                error!("[AppState] Force shutdown failed: {}", e);
            }
        }

        status_tx.send(AppStateStatus::Stopped).ok();
    }
}

impl AppState {
    pub async fn conn_info(&self) -> HashMap<Uuid, ClientInfo> {
        self.session.conn_info().await
    }
}