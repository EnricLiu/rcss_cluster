use std::path::Path;
use common::types::Side;
use crate::config::{BotConfig, ImageQuery, ServerConfig};

mod schema;
mod policy;
mod config;
mod image;

#[tokio::main]
async fn main() {
    let registry = policy::PolicyRegistry::new("sidecars/match_composer/hub");

    let image = ImageQuery {
        provider: "HELIOS".to_string(),
        model: "helios-base".to_string(),
    };

    let server = ServerConfig::default();

    let bot = registry.fetch_bot(BotConfig {
        unum: 0,
        side: Side::LEFT,
        team: "TEST",
        image: &image,
        server: &server,
        log_path: &Path::new("logs/bot.log"),
    }).unwrap();

    let process = bot.spawn().await;
    let mut watch = process.status_watch();
    loop {
        if let Err(e) = watch.changed().await {
            eprintln!("Bot process status watch error: {e}");
            break;
        }

        let status = watch.borrow().clone();
        println!("Bot process status changed: {status:?}");
    }

}

pub struct MatchComposer {

}

impl MatchComposer {
    pub fn from_schema(schema: schema::v1::ConfigV1) -> Result<Self, ()> {
        let host = schema.host;
        let port = schema.port;
        
        todo!()
    }
}

