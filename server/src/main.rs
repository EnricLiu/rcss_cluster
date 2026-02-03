mod error;
mod http;
mod response;
mod ws;
mod udp;
mod state;

use std::env;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use axum::Router;
use clap::Parser;
use log::{debug, info};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use service::Service;
use crate::state::AppState;

pub const PEER_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

#[derive(Parser, Debug)]
#[clap(author = "EnricLiu")]
struct Args {
    #[clap(long, default_value = "0.0.0.0", help = "Server IP to bind")]
    ip: IpAddr,
    #[clap(long, default_value_t = 55555, help = "Server port to bind")]
    port: u16,

    #[clap(flatten)]
    service_args: service::Args,
}

impl Args {
    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
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

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "debug") }
    env_logger::init();

    let args = Args::parse();
    let listen_addr = args.listen_addr();
    let service = match Service::from_args(args.service_args).await {
        Ok(svc) => svc,
        Err(e) => {
            eprintln!("[FATAL] Failed to create service from args: {}", e);
            std::process::exit(1);
        }
    };

    let shutdown_signal = Some(service.shutdown_signal());
    let app = listen(listen_addr, service, shutdown_signal).await;
    app.await.unwrap().unwrap();
}
