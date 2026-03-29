use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use agones::ObjectMeta;
use dashmap::DashMap;

use common::errors::BuilderError;
use common::types::Side;

use crate::declarations::{HostPort, PlayerDeclaration, TeamDeclaration, Unum};
use crate::model::team::TeamModel;
use super::{Declaration, Model};
use super::labels::Labels;
use super::annotations::Annotations;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MetaData {
    pub labels: Labels,
    pub annotations: Annotations,
}

impl TryFrom<ObjectMeta> for MetaData {
    type Error = BuilderError;

    fn try_from(meta: ObjectMeta) -> Result<Self, Self::Error> {
        let labels = meta.labels.try_into()?;
        let annotations = meta.annotations.try_into()?;

        let ret = Self {
            labels,
            annotations,
        };

        Ok(ret)
    }
}

impl MetaData {
    pub fn as_declared(&'_ self) -> Declaration<'_, Self> {
        Declaration::new(self)
    }

    pub fn as_model(&'_ self) -> Model<'_, Self> {
        Model::new(self)
    }
}

impl<'a> Declaration<'a, MetaData> {
    pub fn team(&self, side: Side) -> TeamDeclaration {
        Self::parse_team(side, &self.labels, &self.annotations)
    }

    pub fn players(&self, side: Side) -> DashMap<Unum, PlayerDeclaration> {
        self.team(side).players
    }

    pub fn player(&self, side: Side, unum: Unum) -> Option<PlayerDeclaration> {
        self.players(side).get(&unum).map(|entry| entry.value().clone())
    }

    pub fn teams(&self) -> (TeamDeclaration, TeamDeclaration) {
        (self.team(Side::LEFT), self.team(Side::RIGHT))
    }

    fn parse_team(side: Side, labels: &Labels, annotations: &Annotations) -> TeamDeclaration {
        let (labels, team_name) = match side {
            Side::LEFT => (&labels.left, &annotations.team_l),
            Side::RIGHT => (&labels.right, &annotations.team_r),
            _ => unreachable!(),
        };

        let mut team = TeamDeclaration::builder();
        team.with_side(side).with_name(team_name.clone());
        for label in labels.values() {
            team.add_player(label.player.clone());
        }

        team.build().expect(
            &format!(
                "Failed to build team for side {:?} with name {}:\n\tlabels={:#?}\n\tannotations={:#?}",
                side, team_name, labels, annotations
            )
        )
    }
}

impl<'a> Model<'a, MetaData> {
    pub fn team(&self, side: Side, server: HostPort, log: Option<PathBuf>) -> TeamModel {
        let team_decl = self.as_declared().team(side);
        let mut team = TeamModel::builder();
        team.with_declaration(team_decl)
            .with_server(server)
            .with_log_root(log);

        team.build().expect("Failed to build team model")
    }

    pub fn teams(&self, server: HostPort, log: Option<PathBuf>) -> (TeamModel, TeamModel) {
        (self.team(Side::LEFT, server.clone(), log.clone()), self.team(Side::RIGHT, server, log))
    }
}
