use serde::{Deserialize, Serialize};
use super::Position;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct InitState {
    pub ball: Option<Position>,
}
