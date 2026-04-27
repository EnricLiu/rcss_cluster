use super::image::Image;
use super::host_port::HostPort;
use serde::{Deserialize, Serialize};
use std::ops::Deref;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoachBase {
    pub image: Image,
    #[serde(default)] // false
    pub log: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum CoachKind {
    Helios,
    Ssp,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Coach {
    Helios {
        #[serde(flatten)]
        base: CoachBase,
    },
    Ssp {
        #[serde(flatten)]
        base: CoachBase,
        grpc: HostPort,
    }
}

impl Deref for Coach {
    type Target = CoachBase;

    fn deref(&self) -> &Self::Target {
        match self {
            Coach::Helios { base } => base,
            Coach::Ssp { base, .. } => base,
        }
    }
}

impl Coach {
    pub fn kind(&self) -> CoachKind {
        match self {
            Coach::Helios { .. } => CoachKind::Helios,
            Coach::Ssp { .. } => CoachKind::Ssp,
        }
    }
}

impl Into<CoachBase> for Coach {
    fn into(self) -> CoachBase {
        match self {
            Coach::Helios { base } => base,
            Coach::Ssp { base, .. } => base,
        }
    }
}
