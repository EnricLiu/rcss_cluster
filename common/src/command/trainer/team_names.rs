use std::str::FromStr;

use arcstr::{ArcStr, literal};
use serde::{Deserialize, Serialize};
use super::{Command, CommandAny, TrainerCommand};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandTeamNames;
impl Command for CommandTeamNames {
    type Kind = TrainerCommand;
    type Ok = CommandTeamNamesOk;
    type Error = CommandTeamNamesError;

    fn kind(&self) -> Self::Kind { TrainerCommand::TeamNames }

    fn encode(&self) -> ArcStr {
        literal!("(team_names)")
    }

    fn parse_ret_ok(tokens: &[&str]) -> Option<Self::Ok> {
        let parse_team = |team_tokens: &[&str]| {
            if team_tokens.len() != 3 { return None }
            if team_tokens[0] != "team" { return None }

            let team_name = team_tokens[2].to_string();
            match team_tokens[1] {
                "l" => Some((Some(team_name), None)),
                "r" => Some((None, Some(team_name))),
                _ => None,
            }
        };

        let teams = match tokens.len() {
            6 => {
                let team_1 = parse_team(&tokens[0..3])?;
                let team_2 = parse_team(&tokens[3..6])?;
                (team_1.0.or(team_2.0), team_1.1.or(team_2.1))
            },
            3 => parse_team(tokens)?,
            0 => (None, None),
            _ => return None,
        };

        Some(
            CommandTeamNamesOk {
                left: teams.0,
                right: teams.1,
            }
        )
    }

    // never error
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandTeamNamesOk {
    pub left: Option<String>,
    pub right: Option<String>,
}

#[derive(thiserror::Error, Debug)]
pub enum CommandTeamNamesError {}

impl FromStr for CommandTeamNamesError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <CommandTeamNamesError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
