use std::net::Ipv4Addr;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PlayerConfig<'a> {
    pub host: Ipv4Addr,
    pub port: u16,
    pub unum: u8,
    pub team_name: &'a str,
    pub log_path: &'a Path,
}
