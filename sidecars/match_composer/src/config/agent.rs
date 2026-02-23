use std::path::Path;
use common::types::Side;
use super::{ImageQuery, ServerConfig, PlayerProcessConfig, ImageConfig};


#[derive(Clone, Debug)]
pub struct AgentConfig<'a> {
    pub unum: u8,
    pub side: Side,
    pub team: &'a str,
    pub server: &'a ServerConfig,
    pub grpc: &'a ServerConfig,

    pub image: &'a ImageConfig,

    pub log_path: Option<&'a Path>,
}