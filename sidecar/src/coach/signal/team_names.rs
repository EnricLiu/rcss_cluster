use std::str::FromStr;

use arcstr::{ArcStr, literal};
use crate::coach::signal::SignalKind;

pub struct SignalTeamNames;
impl super::Signal for SignalTeamNames {
    type Ok = (Option<String>, Option<String>); // (left_team_name, right_team_name)
    type Error = SignalTeamNamesError;

    fn kind(&self) -> SignalKind {
        SignalKind::TeamNames
    }

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

        Some(teams)
    }

    // never error
}

#[derive(thiserror::Error, Debug)]
pub enum SignalTeamNamesError {}

impl FromStr for SignalTeamNamesError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, <SignalTeamNamesError as FromStr>::Err> {
        match s {
            _ => Err(()),
        }
    }
}
