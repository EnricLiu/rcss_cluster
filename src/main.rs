mod service;
mod controller;
mod model;

use std::env;

#[tokio::main]
async fn main() {
    unsafe { env::set_var("RUST_LOG", "trace") }
    env_logger::init();

    let app = controller::listen("0.0.0.0:55555").await;
    app.await.unwrap().unwrap();
}
