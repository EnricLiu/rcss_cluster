use super::unum::Unum;
use super::image::Image;
use super::host_port::HostPort;
use crate::model::{PlayerKind};
use serde::{Deserialize, Serialize};
use std::ops::Deref;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerBase {
    pub unum: Unum,
    pub image: Image,
    #[serde(default)] // false
    pub goalie: bool,
    #[serde(default)] // false
    pub log: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Player {
    Helios {
        #[serde(flatten)]
        base: PlayerBase,
    },

    Ssp {
        #[serde(flatten)]
        base: PlayerBase,
        grpc: HostPort,
    }
}

impl Deref for Player {
    type Target = PlayerBase;

    fn deref(&self) -> &Self::Target {
        match self {
            Player::Helios { base } => base,
            Player::Ssp { base, .. } => base,
        }
    }
}

impl Player {
    pub fn kind(&self) -> PlayerKind {
        match self {
            Player::Helios { .. } => PlayerKind::Helios,
            Player::Ssp { .. } => PlayerKind::Ssp,
        }
    }
}

impl Into<PlayerBase> for Player {
    fn into(self) -> PlayerBase {
        match self {
            Player::Helios { base } => base,
            Player::Ssp { base, .. } => base,
        }
    }
}
