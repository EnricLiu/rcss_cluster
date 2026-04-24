mod error;
mod http;
mod proxy;
mod state;

use std::env;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use axum::Router;
use clap::Parser;
use log::{debug, error, info, warn};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use service::Service;

use common::axum::response;
use common::utils::logging::{LoggingArgs, init_stdout_logger, init_dual_logger};

use crate::proxy::udp::UdpProxy;
use crate::state::AppState;

pub const PEER_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);

fn init_logging(
    level: &'static str,
    log_args: &LoggingArgs,
    stdio_suffix: Option<PathBuf>
) -> std::io::Result<()> {
    match (log_args.try_resolve_log_root(), stdio_suffix) {
        (Ok(log_root), Some(stdio_suffix)) => {
            let log_file = log_root.join(stdio_suffix);
            if let Err(e) = init_dual_logger(&log_file, level) {
                eprintln!("[FATAL] Failed to initialize logger at {}: {}", log_file.display(), e);
                return Err(e);
            }
        }
        (Ok(_), None) => {
            error!("[FATAL] Stdio log path is required when log root is specified");
            std::process::exit(1);
        }
        (Err(e), Some(stdio_suffix)) => {
            warn!("[Logging] Log root not specified, use relative path for stdio log: {}, Error: {e}", stdio_suffix.display());
            if let Err(e) = init_dual_logger(&stdio_suffix, level) {
                eprintln!("[FATAL] Failed to initialize logger at {}: {}", stdio_suffix.display(), e);
                return Err(e);
            }
        }
        _ => init_stdout_logger(level),
    };
    Ok(())
}

#[derive(Parser, Debug)]
#[clap(author = "EnricLiu")]
struct Args {
    #[clap(long, default_value = "0.0.0.0", env = "SERVER_HOST", help = "Server IP to bind")]
    ip: IpAddr,
    #[clap(long, default_value_t = 6666, env = "SERVER_HTTP_PORT", help = "Server port to bind")]
    http_port: u16,

    #[clap(long, default_value_t = 6657, env = "SERVER_UDP_PORT_PLAYER", help = "UDP Proxy port for players to bind")]
    player_udp_port: u16,
    #[clap(long, default_value_t = 6658, env = "SERVER_UDP_PORT_TRAINER", help = "UDP Proxy port for trainers to bind")]
    trainer_udp_port: u16,
    #[clap(long, default_value_t = 6659, env = "SERVER_UDP_PORT_COACH", help = "UDP Proxy port for coaches to bind")]
    coach_udp_port: u16,

    #[clap(long, env = "SERVER_STDIO_PATH", default_value = "None", help = "Server service log file")]
    stdio_log_path: Option<PathBuf>,

    #[clap(flatten)]
    log_args: LoggingArgs,

    #[clap(flatten)]
    service_args: service::Args,
}

impl Args {
    pub fn listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.http_port)
    }

    pub fn player_udp_listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.player_udp_port)
    }

    pub fn coach_udp_listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.coach_udp_port)
    }

    pub fn trainer_udp_listen_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.trainer_udp_port)
    }
}

fn route(state: AppState) -> Router {
    Router::new()
        .merge(http::route("/", state.clone()))
        .merge(proxy::ws::route("/player", state))
        .route_layer(TraceLayer::new_for_http())
}

pub async fn listen(
    addr: impl ToSocketAddrs,
    player_prox_udp_addr: impl Into<SocketAddr>,
    service: Service,
    shutdown: Option<impl Future<Output=()> + Send + 'static>
) -> JoinHandle<Result<(), String>> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let state = AppState::new(service, Some(shutdown_rx));

    state.service.spawn().await.expect("FATAL: Service failed to start");

    let listener = TcpListener::bind(addr).await.unwrap();
    let addr = listener.local_addr().unwrap();

    let _state = state.clone();

    let player_prox_udp_addr = player_prox_udp_addr.into();
    tokio::spawn(async move {
        let addr = player_prox_udp_addr;
        match UdpProxy::new(_state, addr).await {
            Ok(proxy) => {
                 info!("[UDP Proxy(Player)] Started on {}", addr);
                 proxy.run().await;
            },
            Err(e) => {
                error!("[UDP Proxy(Player)] Failed to start on {}: {}", addr, e);
            }
        }
    });

    let app = route(state);

    tokio::spawn(async move {
        let serve = axum::serve(listener, app);
        info!("Listening on http://{addr:?}");

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
    let args = Args::parse();

    let listen_addr = args.listen_addr();
    let player_udp_listen_addr = args.player_udp_listen_addr();

    init_logging("info", &args.log_args, args.stdio_log_path).unwrap();
    
    let log_root = args.log_args.try_resolve_log_root().ok().unwrap_or(env::current_dir().unwrap());
    let service = match Service::from_args(args.service_args, log_root).await {
        Ok(svc) => svc,
        Err(e) => {
            eprintln!("[FATAL] Failed to create service from args: {}", e);
            std::process::exit(1);
        }
    };

    let shutdown_signal = Some(service.shutdown_signal());
    let app = listen(listen_addr, player_udp_listen_addr, service, shutdown_signal).await;
    app.await.unwrap().unwrap();
}
