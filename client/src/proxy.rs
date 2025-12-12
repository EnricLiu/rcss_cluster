use std::sync::Arc;
use dashmap::DashMap;
use log::debug;
use reqwest::Url;
use crate::agones::AgonesClient;
use crate::room::{Room, RoomConfig, RoomInfo, WsConfig};
use crate::utils::local_addr;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct ProxyServerConfig {
    // pub agones_config: AgonesConfig,
    // pub api_config: ApiConfig,
    pub agones_url: Url,
}

impl Default for ProxyServerConfig {
    fn default() -> Self {
        Self {
            agones_url: "http://localhost:8080".parse().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct ProxyServer {
    pub config: ProxyServerConfig,
    pub agones: AgonesClient,
    rooms: DashMap<String, Arc<Room>>,
}

impl ProxyServer {
    pub fn new(config: ProxyServerConfig) -> Self {
        let agones = AgonesClient::new(config.agones_url.clone());
        Self {
            config,
            agones,
            rooms: DashMap::new(),
        }
    }

    pub async fn create_room(&self, room_name: String, udp_port: u16) -> Result<RoomConfig> {
        let ws_url = self.agones.allocate().await.expect("Failed to allocate room");

        let config = {
            let ws_config = {
                let mut builder = WsConfig::builder();
                builder.with_base_url(ws_url);
                builder.build_into()
            };

            let mut builder = RoomConfig::builder();
            builder
                .with_ws(ws_config)
                .with_name(room_name.clone())
                .with_player_udp(local_addr(udp_port))
                .build();
            builder.build_into()
        };

        let room = Room::listen(config).await?;
        let room = Arc::new(room);
        self.rooms.insert(room_name.clone(), room);

        if  let Some(room) = self.rooms.get(&room_name) {
            Ok(room.config().clone())
        } else {
            Err(Error::RoomDropped { room_name })
        }
    }

    pub async fn drop_room(&self, room_name: &str) -> Result<()> {
        let room = match self.rooms.remove(room_name) {
            Some((_, room)) => room,
            None => return Err(Error::RoomNotFound { room_name: room_name.to_string() }),
        };

        let retry = 5;
        let room_name = room.cfg.name.clone();

        let mut room = Some(room);
        for _ in 0..retry {
            if let Some(arc) = room {
                room = match Arc::try_unwrap(arc) {
                    Ok(r) => {
                        drop(r);
                        debug!("Room[{room_name}] successfully dropped.");
                        return Ok(())
                    },
                    Err(arc) => Some(arc),
                }
            } else { break; }

        };

        let room = room.expect("WTF? Room dropped but still referenced");

        // failed to get ownership, try insert it back
        let mut insert_ok = false;
        let room_ = room.clone();
        self.rooms.entry(room_name.to_string())
            .or_insert_with(|| { insert_ok = true; room_ });

        match insert_ok {
            true => Err(Error::RoomDropRetrieved { room_name }),
            false => Err(Error::RoomDropDangled { room_name, room }),
        }
    }

    pub fn room(&self, room_name: &str) -> Result<Arc<Room>> {
        self.rooms.get(room_name).map(|r| r.value().clone())
            .ok_or(Error::RoomNotFound { room_name: room_name.to_string() })
    }
    
    pub fn room_info(&self, room_name: &str) -> Result<RoomInfo> {
        self.rooms.get(room_name).map(|r| r.info())
            .ok_or(Error::RoomNotFound { room_name: room_name.to_string() })
    }

    pub fn all_room_infos(&self) -> Vec<RoomInfo> {
        self.rooms.iter().map(|r| r.info()).collect()
    }
    
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        for room in self.rooms.iter() {
            self.drop_room(room.key()).await?;
        }
        Ok(())
    }
}
