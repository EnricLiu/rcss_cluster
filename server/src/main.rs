mod controller;

use std::env;
use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use service::Service;

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

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "trace") }
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
    let app = controller::listen(listen_addr, service, shutdown_signal).await;
    app.await.unwrap().unwrap();
}
