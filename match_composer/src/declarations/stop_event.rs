use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct StoppingEvent {
    pub timeup: Option<u16>,
}

