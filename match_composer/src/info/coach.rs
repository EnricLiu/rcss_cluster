use serde::{Deserialize, Serialize};
use common::process::ProcessStatusSerDes;

use crate::declaration::ImageDeclaration;
use crate::model::CoachKind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoachInfo {
    pub kind: CoachKind,
    pub status: CoachStatusInfo,
    pub image: ImageDeclaration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CoachStatusInfo {
    Unknown,
    #[serde(untagged)]
    Some(ProcessStatusSerDes),
}
