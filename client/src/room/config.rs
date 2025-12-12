use std::net::SocketAddr;
use std::time::Duration;
use reqwest::Url;
use serde::ser::SerializeMap;
use serde::Serialize;
use crate::utils::local_addr;

#[derive(Serialize, Clone, Debug)]
pub struct RoomConfig {
    pub name: String,
    pub ws: WsConfig,
    pub player_udp: SocketAddr,
    pub trainer_udp: SocketAddr,
    pub coach_udp: SocketAddr,
}

impl Default for RoomConfig {
    fn default() -> Self {
        RoomConfig {
            name: "default room".to_string(),
            ws: WsConfig::default(),
            player_udp: local_addr(6000),
            trainer_udp: local_addr(6001),
            coach_udp: local_addr(6002),
        }
    }
}

impl RoomConfig {
    pub fn builder() -> RoomConfigBuilder {
        RoomConfigBuilder::new()
    }
}

#[derive(Clone, Default, Debug)]
pub struct RoomConfigBuilder {
    name: Option<String>,
    ws: Option<WsConfig>,
    player_udp: Option<SocketAddr>,
    trainer_udp: Option<SocketAddr>,
    coach_udp: Option<SocketAddr>,
}

impl RoomConfigBuilder {
    fn new() -> Self {
        RoomConfigBuilder {
            name: None,
            ws: None,
            player_udp: None,
            trainer_udp: None,
            coach_udp: None,
        }
    }

    pub fn with_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn with_ws(&mut self, ws: WsConfig) -> &mut Self {
        self.ws = Some(ws);
        self
    }

    pub fn with_player_udp(&mut self, player_udp: SocketAddr) -> &mut Self {
        self.player_udp = Some(player_udp);
        self
    }

    pub fn with_trainer_udp(&mut self, trainer_udp: SocketAddr) -> &mut Self {
        self.trainer_udp = Some(trainer_udp);
        self
    }

    pub fn with_coach_udp(&mut self, coach_udp: SocketAddr) -> &mut Self {
        self.coach_udp = Some(coach_udp);
        self
    }

    pub fn build_into(self) -> RoomConfig {
        let mut ret = RoomConfig::default();
        if let Some(name) = self.name {
            ret.name = name;
        }
        if let Some(ws) = self.ws {
            ret.ws = ws;
        }
        if let Some(player_udp) = self.player_udp {
            ret.player_udp = player_udp;
        }
        if let Some(trainer_udp) = self.trainer_udp {
            ret.trainer_udp = trainer_udp;
        }
        if let Some(coach_udp) = self.coach_udp {
            ret.coach_udp = coach_udp;
        }

        ret
    }

    pub fn build(&self) -> RoomConfig {
        self.clone().build_into()
    }
}


#[derive(Clone, Debug)]
pub struct WsConfig {
    pub base_url: Url,
    pub reconnect_delay: Duration,
    pub max_reconnect_attempts: u32,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("ws://default.ws.url/").unwrap(),
            reconnect_delay: Duration::from_millis(500),
            max_reconnect_attempts: 5,
        }
    }
}

impl Serialize for WsConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("base_url", &self.base_url.to_string())?;
        map.serialize_entry("reconnect_delay", &self.reconnect_delay.as_millis())?;
        map.serialize_entry("max_reconnect_attempts", &self.max_reconnect_attempts)?;
        map.end()
    }
}

impl WsConfig {
    pub fn builder() -> WsConfigBuilder {
        WsConfigBuilder::new()
    }
}


#[derive(Clone, Default, Debug)]
pub struct WsConfigBuilder {
    base_url: Option<Url>,
    reconnect_delay: Option<Duration>,
    max_reconnect_attempts: Option<u32>,
}

impl WsConfigBuilder {
    fn new() -> Self {
        WsConfigBuilder {
            base_url: None,
            reconnect_delay: None,
            max_reconnect_attempts: None,
        }
    }

    pub fn with_base_url(&mut self, base_url: Url) -> &mut Self {
        self.base_url = Some(base_url);
        self
    }

    pub fn with_reconnect_delay(&mut self, reconnect_delay: Duration) -> &mut Self {
        self.reconnect_delay = Some(reconnect_delay);
        self
    }

    pub fn with_max_reconnect_attempts(&mut self, max_reconnect_attempts: u32) -> &mut Self {
        self.max_reconnect_attempts = Some(max_reconnect_attempts);
        self
    }

    pub fn build_into(self) -> WsConfig {
        let mut ret = WsConfig::default();
        if let Some(base_url) = self.base_url {
            ret.base_url = base_url;
        }
        if let Some(reconnect_delay) = self.reconnect_delay {
            ret.reconnect_delay = reconnect_delay;
        }
        if let Some(max_reconnect_attempts) = self.max_reconnect_attempts {
            ret.max_reconnect_attempts = max_reconnect_attempts;
        }
        ret
    }

    pub fn build(&self) -> WsConfig {
        self.clone().build_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_room_config_builder() {
        let room_config = RoomConfig::builder()
            .with_name("test room".to_string())
            .with_ws(WsConfig::builder()
                .with_base_url(Url::parse("ws://test.ws.url/").unwrap())
                .with_reconnect_delay(Duration::from_millis(114514))
                .with_max_reconnect_attempts(1919810)
                .build())
            .with_player_udp(local_addr(6657))
            .with_trainer_udp(local_addr(5555))
            .with_coach_udp(local_addr(6666))
            .build();
        assert_eq!(room_config.name, "test room");
        assert_eq!(room_config.ws.base_url, Url::parse("ws://test.ws.url/").unwrap());
        assert_eq!(room_config.ws.reconnect_delay, Duration::from_millis(114514));
        assert_eq!(room_config.ws.max_reconnect_attempts, 1919810);
        assert_eq!(room_config.player_udp, local_addr(6657));
        assert_eq!(room_config.trainer_udp, local_addr(5555));
        assert_eq!(room_config.coach_udp, local_addr(6666));
    }
}
