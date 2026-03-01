use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

