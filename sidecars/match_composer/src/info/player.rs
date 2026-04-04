use serde::Serialize;
use crate::model::{PlayerKind};
use crate::declarations::{ImageDeclaration, Unum};
use common::process::ProcessStatusSerializable;

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
