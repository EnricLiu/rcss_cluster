mod process;
mod coach;
mod sidecar;

#[cfg(target_os = "windows")]
compile_error!("This program currently not supported on Windows.");

use std::env;
use env_logger;

// #[tokio::main]
// async fn main() {
//     unsafe { env::set_var("RUST_LOG", "trace") };
//     env_logger::init();
//
//     let mut builder = process::ServerProcess::spawner("rcssserver").await;
//     use itertools::Itertools;
//     let mut ports = (6000..).chunks(3);
//     for (server, coach, sidecar) in ports {
//         builder.with_ports(server, coach, sidecar);
//         let process = builder.spawn().await.unwrap();
//         println!("Process running, pid = {:?}", process.pid());
//         let ret = process.terminate().await.unwrap();
//         println!("Process terminated, ret code = {ret}")
//     }
//
// }

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "trace") };
    env_logger::init();

    let builder = process::ServerProcess::spawner("rcssserver").await;
    let process = builder.spawn().await.unwrap();
    println!("Process running, pid = {:?}", process.pid());

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let ret = process.terminate().await.unwrap();
    println!("Process terminated, ret code = {ret}")

}
