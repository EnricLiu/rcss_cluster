use serde::{Serialize, Deserialize};

use super::Status;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamInfo {
    pub n_client: usize,
    pub name: String,
    pub status: Status,
}