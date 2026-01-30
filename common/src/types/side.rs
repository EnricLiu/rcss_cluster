use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(C)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    LEFT = 1,
    NEUTRAL = 0,
    RIGHT = -1
}
