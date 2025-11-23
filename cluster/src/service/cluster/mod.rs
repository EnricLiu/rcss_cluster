pub mod error;

use std::sync::{Arc, Weak};
use dashmap::DashMap;
use dashmap::mapref::one::Ref;
use uuid::Uuid;

use super::team::{self, Team};
use super::room::{self, Room};
use super::client::{self, Client};

use crate::model::room::Info as RoomInfo;

use error::*;
pub use error::{Error, Result};

#[derive(Debug)]
pub struct Cluster {
    rooms: DashMap<Uuid, Room>,
    clients: DashMap<Uuid, Weak<Client>>,
}

impl Cluster {
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            clients: DashMap::new(),
        }
    }

    pub fn create_room(&self, config: room::Config) -> Uuid {
        let id = Uuid::now_v7();
        self.rooms.insert(id, Room::new(config));
        id
    }

    pub fn drop_room(&self, room_id: Uuid) -> Result<()> {
        if self.rooms.remove(&room_id).is_none() {
            return RoomNotFoundSnafu { room_id }.fail();
        }
        Ok(())
    }

    pub async fn room_info(&self, room_id: Uuid) -> Result<RoomInfo> {
        let room = self.room(room_id)?;
        Ok(room.info().await)
    }

    pub async fn all_rooms_info(&self) -> Result<Vec<RoomInfo>> {
        let mut infos = Vec::with_capacity(self.rooms.len());
        for room in self.rooms.iter() {
            infos.push(room.info().await);
        }

        Ok(infos)
    }

    fn room(&'_ self, room_id: Uuid) -> Result<Ref<'_, Uuid, Room>> {
        match self.rooms.get(&room_id) {
            Some(room) => Ok(room),
            None => RoomNotFoundSnafu { room_id }.fail(),
        }
    }

    pub async fn create_team(&self, room_id: Uuid, config: team::Config) -> Result<String> {
        let team_name = self.room(room_id)?.add_team(None, config).await?;
        Ok(team_name)
    }

    pub async fn create_client(&self, room_id: Uuid, team_name: String, config: client::Config) -> Result<Uuid> {
        let team_name = &team_name;
        let (id, client) = self.room(room_id)?.with_team(
            team_name, async move |team| {
                let team = team.ok_or(TeamNotFoundSnafu {
                    room_id, team_name: team_name.to_string()
                }.build())?;

                let id = Uuid::now_v7();
                let client = Arc::new(Client::new(config.clone()));
                let client_ = Arc::downgrade(&client);
                team.add_client(id, Arc::clone(&client)).await?;

                Ok::<_, Error>((id, client_))
            },
        ).await?;

        self.clients.insert(id, client);
        Ok(id)
    }
    
    pub async fn client(&self, client_id: Uuid) -> Result<Arc<Client>> {
        match self.clients.get(&client_id) {
            Some(client) => {
                client.value().upgrade().ok_or(ClientReleasedSnafu { client_id }.build())
            },
            None => ClientNotFoundSnafu { client_id }.fail(),
        }
    }

    // pub async fn client_send(&self, client_id: Uuid, message: impl rcss::Message) -> Result<()> {
    //     let client = self.clients.get(&client_id)
    //         .ok_or(ClientNotFoundSnafu { client_id }.build())?;
    // 
    //     client.send(message).await?;
    //     Ok(())
    // }
}