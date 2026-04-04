use serde::Serialize;
use common::process::ProcessStatusSerializable;

use crate::model::{PlayerKind};
use crate::declaration::{ImageDeclaration, Unum};


#[derive(Serialize, Debug, Clone)]
pub struct PlayerInfo {
    pub unum: Unum,
    pub kind: PlayerKind,
    pub status: PlayerStatusInfo,
    pub image: ImageDeclaration,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatusInfo {
    Unknown,
    #[serde(untagged)]
    Some(ProcessStatusSerializable),
}
