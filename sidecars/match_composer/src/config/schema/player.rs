use crate::schema::v1::{PlayerV1, PolicyV1};

#[derive(Clone, Debug)]
pub struct PlayerSchema {
    pub unum: u8,
    pub goalie: bool,
    pub policy: PlayerPolicySchema,
}

#[derive(Clone, Debug)]
pub enum PlayerPolicySchema {
    Bot {
        image: String,
    },
    Agent {
        image: String,
        grpc_host: std::net::Ipv4Addr,
        grpc_port: u16,
    },
}

impl From<PlayerV1> for PlayerSchema {
    fn from(player: PlayerV1) -> Self {
        let policy = match player.policy {
            PolicyV1::Bot { image } => PlayerPolicySchema::Bot { image },
            PolicyV1::Agent(agent) => PlayerPolicySchema::Agent {
                image: agent.image().to_string(),
                grpc_host: agent.grpc_host(),
                grpc_port: agent.grpc_port(),
            },
        };

        PlayerSchema {
            unum: player.unum,
            goalie: player.goalie,
            policy,
        }
    }
}

