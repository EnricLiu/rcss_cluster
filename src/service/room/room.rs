use std::sync::{Arc, LazyLock};
use std::collections::{HashMap, HashSet};
use std::ops::Sub;
use log::{debug, trace, warn};

use uuid::Uuid;
use tokio::sync::RwLock;
use dashmap::DashMap;
use dashmap::mapref::one::Ref;

use crate::model::room;
use crate::service::team::{Team, Config as TeamConfig, Side as TeamSide};
use crate::service::client::Client;

use super::{AtomicStatus, Config};
use super::error::*;

#[derive(Default, Debug)]
pub struct Room {
    id:         Uuid,
    config:     Config,

    teams:      RwLock<DashMap<String, Team>>,
    trainer:    DashMap<Uuid, Arc<Client>>,

    status:     Arc<AtomicStatus>,
}

impl Room {
    pub const NUM_TEAMS: usize = 2;
    pub const SIDES: LazyLock<HashSet<TeamSide>> = LazyLock::new(
        || HashSet::from([TeamSide::Left, TeamSide::Right, ])
    );

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

    pub async fn info(&self) -> room::Info {
        let (team_l, team_r) = {
            let teams = self.teams.read().await;
            let map: HashMap<_,_> = teams.iter().map(|team| (team.side(), team)).collect();
            let team_l = match map.get(&TeamSide::Left) {
                Some(team) => Some(team.value().info().await),
                None => None,
            };
            let team_r = match map.get(&TeamSide::Right) {
                Some(team) => Some(team.value().info().await),
                None => None,
            };
            (team_l, team_r)
        };

        room::Info {
            room_id: self.id,
            name: self.name().to_string(),
            team_l,
            team_r,
            status: self.status.kind(),
        }
    }

    pub async fn add_team(&self, side: Option<TeamSide>, config: TeamConfig) -> Result<String> {
        if self.teams.read().await.len() >= Self::NUM_TEAMS {
            debug!("Room[{}]: is Full, cannot add team {}", self.name(), config.name);
            return RoomIsFullSnafu {
                room_name: self.name().to_string(),
                pending_team: config.name.to_string(),
            }.fail();
        }

        let team_name = { // [self.teams] WRITE LOCK
            trace!("Room[{}]: trying the lock: `self.teams`", self.name());
            let teams_guard = self.teams.write().await;
            trace!("Room[{}]: got the lock: `self.teams`", self.name());

            let side = {
                let mut all_side = {
                    let mut all_side = HashMap::new();
                    for team in teams_guard.iter() {
                        let team_name = team.name().to_string();
                        if config.name == team_name {
                            debug!("Room[{}]: team {} already exists", self.name(), team_name);
                            return RoomNameOccupiedSnafu {
                                room_name: self.name().to_string(),
                                team_name: team_name.to_string(),
                            }.fail();
                        }
                        all_side.insert(team.side(), team.name().to_string());
                    }
                    all_side
                };

                trace!("Room[{}]: all_sides: {:?}", self.name(), all_side);

                if all_side.len() >= Self::NUM_TEAMS { // racing happened
                    warn!("Room[{}]: is Full, cannot add team {}", self.name(), config.name);
                    return RoomIsFullSnafu {
                        room_name: self.name().to_string(),
                        pending_team: config.name,
                    }.fail();
                }

                match side {
                    Some(side) => {
                        if let Some(occupied_team) = all_side.remove(&side) {
                            return RoomSideOccupiedSnafu {
                                room_name: self.name().to_string(),
                                pending_team: config.name,
                                occupied_team,
                                target_side: side,
                            }.fail();
                        }
                        side
                    }
                    None => {
                        let side_remain = Self::SIDES.sub(&all_side.into_keys().collect());
                        side_remain.into_iter().next().expect("No side available")
                    }
                }
            };

            let team_name = config.name.clone();

            debug!("Room[{}]: adding team {} to side {}", self.name(), team_name, side);

            teams_guard.insert(
                team_name.clone(),
                Team::new(side, config),
            );

            trace!("Room[{}]: `self.teams` lock released", self.name());

            team_name
        }; // [self.teams] WRITE RELEASE

        Ok(team_name)
    }
    
    pub async fn with_team<R: Send + 'static>(&self, team_name: &str, f: impl AsyncFn(Option<Ref<String, Team>>) -> R) -> R {
        let teams_guard = self.teams.read().await;
        f(teams_guard.get(team_name)).await
    }
}