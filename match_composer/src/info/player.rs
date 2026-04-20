use serde::{Deserialize, Serialize};
use common::process::ProcessStatusSerDes;

use crate::model::{PlayerKind};
use crate::declaration::{ImageDeclaration, Unum};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub unum: Unum,
    pub kind: PlayerKind,
    pub status: PlayerStatusInfo,
    pub image: ImageDeclaration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatusInfo {
    Unknown,
    #[serde(untagged)]
    Some(ProcessStatusSerDes),
}
