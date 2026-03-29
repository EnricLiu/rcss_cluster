use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

