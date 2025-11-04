use std::sync::Arc;
use std::collections::HashMap;
use log::{debug, trace, warn};

use uuid::Uuid;
use tokio::sync::RwLock;
use dashmap::DashMap;


use crate::service::team::{Team, Config as TeamConfig, Side as TeamSide};
use crate::service::udp::UdpConnection;

use super::{Status, Config};
use super::error::*;

pub const NUM_TEAMS: usize = 2;

#[derive(Default, Debug)]
pub struct Room {
    id: Uuid,
    config:     Config,

    teams:      RwLock<DashMap<String, Team>>,
    trainer:    DashMap<Uuid, Arc<UdpConnection>>,

    status:     Status,
}

impl Room {
    pub fn new(config: Config) -> Self {
        Room {
            config,
            id: Uuid::now_v7(),
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub async fn add_team(&self, side: Option<TeamSide>, config: TeamConfig) -> Result<()> {
        if self.teams.read().await.len() >= NUM_TEAMS {
            debug!("Room[{}]: is Full, cannot add team {}", self.name(), config.name);
            return RoomIsFullSnafu {
                room_name: self.name().to_string(),
                pending_team: config.name.to_string(),
            }.fail();
        }

        { // [self.teams] WRITE LOCK
            trace!("Room[{}]: trying the lock: `self.teams`", self.name());
            let teams_guard = self.teams.write().await;
            trace!("Room[{}]: got the lock: `self.teams`", self.name());

            let side = {
                let mut all_side = {
                    let mut all_side = HashMap::new();
                    for team in teams_guard.iter() {
                        all_side.insert(team.side(), team.name().to_string());
                    }
                    all_side
                };

                trace!("Room[{}]: all_sides: {:?}", self.name(), all_side);

                if all_side.len() >= NUM_TEAMS {
                    warn!("Room[{}]: is Full, cannot add team {}", self.name(), config.name);
                    return RoomIsFullSnafu {
                        room_name: self.name().to_string(),
                        pending_team: config.name.to_string(),
                    }.fail();
                }

                match side {
                    Some(side) => {
                        if let Some(occupied_team) = all_side.remove(&side) {
                            return RoomSideOccupiedSnafu {
                                room_name: self.name().to_string(),
                                pending_team: config.name.to_string(),
                                occupied_team,
                                target_side: side,
                            }.fail();
                        }
                        side
                    }
                    None => {
                        all_side.into_iter().next().expect("No side available").0
                    }
                }
            };

            debug!("Room[{}]: adding team {} to side {}", self.name(), config.name, side);

            teams_guard.insert(
                config.name.to_string(),
                Team::new(side, config),
            );

            trace!("Room[{}]: `self.teams` lock released", self.name());
        } // [self.teams] WRITE RELEASE

        Ok(())
    }
}