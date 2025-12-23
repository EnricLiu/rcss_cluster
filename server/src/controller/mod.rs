mod error;
mod http;
mod response;
mod ws;

use std::pin::Pin;
use std::sync::{Arc, Weak};

use axum::Router;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use dashmap::DashMap;
use chrono::{Utc, Duration};
use log::{debug, error, info};

pub use error::Error;
pub use response::Response;

use tower_http::trace::TraceLayer;
use uuid::Uuid;

use service::Service;

#[derive(Clone)]
pub struct AppState {
    service: Arc<Service>,
    players: Arc<DashMap<Uuid, Weak<common::client::Client>>>,
}

impl AppState {
    pub const CLEANER_POLL_INTERVAL: Duration = Duration::seconds(1);
    pub const CLEANER_TIMEOUT: Duration = Duration::seconds(30);
    
    pub fn new(service: Service, shutdown_notifier: Option<oneshot::Receiver<()>>) -> Self {
        let service = Arc::new(service);
        
        if let Some(shutdown_notifier) = shutdown_notifier {
            tokio::spawn(Self::run_wait_for_shutdown_cleaner(
                service.clone(), shutdown_notifier));
        }
        
        Self {
            service,
            players: Arc::new(DashMap::new()),
        }
    }
    
    async fn run_wait_for_shutdown_cleaner(
        mut service: Arc<Service>,
        shutdown_notifier: oneshot::Receiver<()>,
    ) {
        shutdown_notifier.await.ok();
        debug!("[AppState] Shutdown notifier received, starting polling service shutdown...");
        
        let mut interval = tokio::time::interval(Self::CLEANER_POLL_INTERVAL.to_std().unwrap());
        let start_at = Utc::now();
        
        loop {
            interval.tick().await;
            
            if let Some(service) = Arc::get_mut(&mut service) {
                match service.shutdown().await {
                    Ok(_) => {
                        info!("[AppState] Service shutdown completed in {}ms.", 
                            (Utc::now() - start_at).num_milliseconds());
                        break;
                    },
                    Err(e) => {
                        error!("[AppState] Service shutdown failed: {}. Retrying...", e);
                    },
                }
            }
            
            if (Utc::now() - start_at) > Self::CLEANER_TIMEOUT {
                error!("[AppState] Service shutdown timed out after {} seconds.", 
                    Self::CLEANER_TIMEOUT.num_seconds());
                break;
            }
        }
    }
}

fn route(state: AppState) -> Router {
    Router::new()
        .merge(http::route("/", state.clone()))
        .merge(ws::route("/player", state))
        .route_layer(TraceLayer::new_for_http())
}

pub async fn listen(
    addr: impl ToSocketAddrs,
    service: Service,
    shutdown: Option<impl Future<Output=()> + Send + 'static>
) -> JoinHandle<Result<(), String>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = AppState::new(service, Some(shutdown_rx));

    state.service.spawn().await.expect("FATAL: Service failed to start");

    let app = route(state);
    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let serve = axum::serve(listener, app);
        info!("Listening on http://{addr}");

        let shutdown: Pin<Box<dyn Future<Output=()> + Send>> = match shutdown {
            Some(signal) => Box::pin(signal),
            None => Box::pin(futures::future::pending::<()>()),
        };

        let signal = async {
            tokio::select! {
                _ = shutdown => {
                    debug!("[Server] Shutdown signal received, shutting down...");
                },
                _ = tokio::signal::ctrl_c() => {
                    debug!("[Server] Ctrl-C received, shutting down...");
                },
            }

            let _ = shutdown_tx.send(());
            debug!("[Server] Shutdown signal sent to AppState cleaner");
        };

        serve.with_graceful_shutdown(signal).await.map_err(|e| e.to_string())
    })
}
