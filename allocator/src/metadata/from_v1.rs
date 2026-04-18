use std::collections::HashMap;

use common::errors::BuilderError;

use crate::declaration::{
    HostPort,
    ImageDeclaration,
    InitStateDeclaration,
    PlayerBaseDeclaration,
    PlayerDeclaration,
    Position as PositionDeclaration,
    RefereeDeclaration,
    StopEventDeclaration,
    Unum,
};
use crate::schema::{
    Schema,
    v1::{ConfigV1, PlayerV1, PolicyV1, TeamSideV1, TeamV1},
};

use super::{Annotations, Labels, MetaData, labels::PlayerLabel};

impl TryFrom<ConfigV1> for MetaData {
    type Error = BuilderError;

    fn try_from(value: ConfigV1) -> Result<Self, Self::Error> {
        value.verify().map_err(|message| BuilderError::InvalidField {
            field: "config",
            message: message.to_string(),
        })?;

        let ConfigV1 {
            teams,
            referee,
            stopping,
            init_state,
            log,
            ..
        } = value;

        let (team_l, players_l) = parse_team(TeamSideV1::Left, teams.left, log)?;
        let (team_r, players_r) = parse_team(TeamSideV1::Right, teams.right, log)?;

        Ok(MetaData {
            labels: Labels::new(players_l, players_r),
            annotations: Annotations {
                team_l,
                team_r,
                referee: RefereeDeclaration {
                    enabled: referee.enable,
                },
                stopping: StopEventDeclaration {
                    timeup: stopping.time_up,
                },
                init: InitStateDeclaration {
                    ball: init_state.ball.map(|position| PositionDeclaration {
                        x: position.x.into(),
                        y: position.y.into(),
                    }),
                },
            },
        })
    }
}

fn parse_team(
    side: TeamSideV1, team: TeamV1, log: bool
) -> Result<(String, HashMap<Unum, PlayerLabel>), BuilderError> {
    let TeamV1 {
        name,
        side: team_side,
        players,
    } = team;

    if side != team_side {
        return Err(BuilderError::InvalidValue {
            field: "team side",
            value: team_side.to_string(),
            expected: side.to_string(),
        })
    }

    let mut labels = HashMap::new();

    for player in players {
        let (unum, label) = convert_player(player, log)?;
        if labels.insert(unum, label).is_some() {
            return Err(BuilderError::InvalidField {
                field: "teams.players",
                message: format!("duplicate player unum {unum} for team {name}"),
            });
        }
    }

    Ok((name, labels))
}

fn convert_player(player: PlayerV1, log: bool) -> Result<(Unum, PlayerLabel), BuilderError> {
    let PlayerV1 {
        unum,
        goalie,
        policy,
        ..
    } = player;

    let unum = Unum::try_from(unum)?;
    let image = ImageDeclaration::try_from(policy.image().to_string())?;
    let base = PlayerBaseDeclaration {
        unum,
        image,
        goalie,
        log,
    };

    let player = match policy {
        PolicyV1::Bot { .. } => PlayerDeclaration::Helios { base },
        PolicyV1::Agent(agent) => PlayerDeclaration::Ssp {
            base,
            grpc: HostPort {
                host: agent.grpc_host(),
                port: agent.grpc_port(),
            },
        },
    };

    Ok((unum, PlayerLabel { player }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn converts_config_into_metadata_by_side() {
        let config: ConfigV1 = serde_json::from_value(json!({
            "host": "127.0.0.1",
            "port": 6000,
            "referee": { "enable": false },
            "stopping": { "time_up": 6000, "goal_l": 3 },
            "init_state": {
                "ball": { "x": 0.5, "y": 0.25 }
            },
            "teams": {
                "allies": {
                    "name": "Righties",
                    "side": "right",
                    "players": [{
                        "unum": 1,
                        "goalie": true,
                        "policy": {
                            "kind": "agent",
                            "agent": "ssp",
                            "image": "Cyrus2D/SoccerSimulationProxy",
                            "grpc_host": "127.0.0.1",
                            "grpc_port": 6657
                        }
                    }]
                },
                "opponents": {
                    "name": "Lefties",
                    "side": "left",
                    "players": [{
                        "unum": 2,
                        "goalie": false,
                        "policy": {
                            "kind": "bot",
                            "image": "HELIOS/helios-base"
                        }
                    }]
                }
            }
        }))
        .expect("config should deserialize");

        let metadata = MetaData::try_from(config).expect("config should convert");
        let left_unum = Unum::try_from(2).expect("unum should be valid");
        let right_unum = Unum::try_from(1).expect("unum should be valid");

        assert_eq!(metadata.annotations.team_l, "Lefties");
        assert_eq!(metadata.annotations.team_r, "Righties");
        assert!(!metadata.annotations.referee.enabled);
        assert_eq!(metadata.annotations.stopping.timeup, Some(6000));

        let ball = metadata
            .annotations
            .init
            .ball
            .expect("ball init state should be preserved");
        assert_eq!(ball.x, 0.5);
        assert_eq!(ball.y, 0.25);

        assert!(matches!(
            &metadata.labels.left[&left_unum].player,
            PlayerDeclaration::Helios { .. }
        ));
        assert!(matches!(
            &metadata.labels.right[&right_unum].player,
            PlayerDeclaration::Ssp { .. }
        ));
    }

    #[test]
    fn rejects_duplicate_player_unums_within_team() {
        let config: ConfigV1 = serde_json::from_value(json!({
            "teams": {
                "allies": {
                    "name": "HB1",
                    "players": [
                        {
                            "unum": 1,
                            "policy": {
                                "kind": "bot",
                                "image": "HELIOS/helios-base"
                            }
                        },
                        {
                            "unum": 1,
                            "policy": {
                                "kind": "bot",
                                "image": "HELIOS/helios-base"
                            }
                        }
                    ]
                },
                "opponents": {
                    "name": "HB2",
                    "players": [{
                        "unum": 2,
                        "policy": {
                            "kind": "bot",
                            "image": "HELIOS/helios-base"
                        }
                    }]
                }
            }
        }))
        .expect("config should deserialize");

        let error = MetaData::try_from(config).expect_err("duplicate unum should fail");

        assert!(matches!(
            error,
            BuilderError::InvalidField {
                field: "teams.players",
                ..
            }
        ));
    }
}