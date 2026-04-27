use std::ops::Deref;
use std::path::PathBuf;
use agones::ObjectMeta;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use allocator::metadata::MetaData as AllocatorMetadata;
use allocator::declaration::{PlayerDeclaration, TeamDeclaration, Unum};
use common::types::Side;

use crate::config::RcssServerConfig;
use crate::model::TeamModel;

use super::{Declaration, Model};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaData {
    #[serde(flatten)]
    meta: AllocatorMetadata,
}

impl TryFrom<ObjectMeta> for MetaData {
    type Error = String;

    fn try_from(meta: ObjectMeta) -> Result<Self, Self::Error> {
        let meta = AllocatorMetadata::try_from(meta).map_err(|e| e.to_string())?;
        Ok(Self { meta })
    }
}

impl From<AllocatorMetadata> for MetaData {
    fn from(meta: AllocatorMetadata) -> Self {
        Self { meta }
    }
}

impl Deref for MetaData {
    type Target = AllocatorMetadata;

    fn deref(&self) -> &Self::Target {
        &self.meta
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
        Self::parse_team(side, self.inner)
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

    fn parse_team(side: Side, meta: &AllocatorMetadata) -> TeamDeclaration {
        let (labels, team_name) = match side {
            Side::LEFT => (&meta.labels.left, &meta.annotations.team_l),
            Side::RIGHT => (&meta.labels.right, &meta.annotations.team_r),
            _ => unreachable!(),
        };
        let coach = match side {
            Side::LEFT => meta.annotations.coach_l.clone(),
            Side::RIGHT => meta.annotations.coach_r.clone(),
            _ => unreachable!(),
        };

        let mut team = TeamDeclaration::builder();
        team.with_side(side).with_name(team_name.clone());
        if let Some(coach) = coach {
            team.with_coach(coach);
        }
        for label in labels.values() {
            team.add_player(label.player.clone());
        }

        team.build().expect(
            &format!(
                "Failed to build team for side {:?} with name {}:\n\tmetadata={:#?}",
                side, team_name, meta
            )
        )
    }
}

impl<'a> Model<'a, MetaData> {
    pub fn team(&self, side: Side, server: RcssServerConfig, player_log_root: Option<PathBuf>) -> TeamModel {
        let team_decl = self.as_declared().team(side);
        let mut team = TeamModel::builder();
        team.with_declaration(team_decl)
            .with_server(server)
            .with_log_root(player_log_root);

        team.build().expect("Failed to build team model")
    }

    pub fn teams(&self, server: RcssServerConfig, player_log_root: Option<PathBuf>) -> (TeamModel, TeamModel) {
        (self.team(Side::LEFT, server.clone(), player_log_root.clone()), self.team(Side::RIGHT, server, player_log_root))
    }
}
