use std::net::{Ipv4Addr, SocketAddr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RoomConfig {
    pub name: String,

    pub host_player_addr:   SocketAddr,
    pub peer_player_addr:   SocketAddr,

    pub host_trainer_addr:  SocketAddr,
    pub peer_trainer_addr:  SocketAddr,

    pub host_coach_addr:    SocketAddr,
    pub peer_coach_addr:    SocketAddr,

    pub num_players:    u8,
    pub num_goalies:    u8,
    pub num_coaches:    u8,
    pub num_trainers:   u32,
}

impl Default for RoomConfig {
    fn default() -> Self {
        let localhost = Ipv4Addr::LOCALHOST.into();
        
        RoomConfig {
            name: "default room".to_string(),

            host_player_addr:   SocketAddr::new(localhost, 0),
            peer_player_addr:   SocketAddr::new(localhost, 6000),
            
            host_trainer_addr:  SocketAddr::new(localhost, 0),
            peer_trainer_addr:  SocketAddr::new(localhost, 6001),

            host_coach_addr:    SocketAddr::new(localhost, 0),
            peer_coach_addr:    SocketAddr::new(localhost, 6002),

            num_players:    10,
            num_goalies:    1,
            num_coaches:    1,
            num_trainers:   u32::MAX,
        }
    }
}