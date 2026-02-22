use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PlayerProcessConfig<'a> {
    pub host: Ipv4Addr,
    pub port: u16,
    pub unum: u8,
    pub goalie: bool,
    pub team_name: &'a str,
    pub log_path: &'a Path,
}
