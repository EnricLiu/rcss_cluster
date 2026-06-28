use serde::{Deserialize, Serialize};
use common::process::ProcessStatusSerDes;

use crate::declaration::ImageDeclaration;
use crate::model::TrainerKind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrainerInfo {
    pub kind: TrainerKind,
    pub status: TrainerStatusInfo,
    pub image: ImageDeclaration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TrainerStatusInfo {
    Unknown,
    #[serde(untagged)]
    Some(ProcessStatusSerDes),
}
