use std::backtrace::Backtrace;

use uuid::Uuid;
use snafu::Snafu;
use strum_macros::IntoStaticStr;

use crate::service::{client, team, room};

#[derive(Snafu, IntoStaticStr, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Cluster: Room[{room_id}] not found."))]
    RoomNotFound {
        room_id: Uuid,
    },
    
    #[snafu(transparent)]
    Room {
        source: room::Error,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Cluster: Team[{team_name}] not found in Room[{room_id}]."))]
    TeamNotFound {
        room_id: Uuid,
        team_name: String,
    },
    
    #[snafu(transparent)]
    Team {
        source: team::Error,
        backtrace: Backtrace,
    },
    
    #[snafu(display("Cluster: Client[{client_id}] not found."))]
    ClientNotFound {
        client_id: Uuid,
    },

    #[snafu(display("Cluster: Client[{client_id}] is already released."))]
    ClientReleased {
        client_id: Uuid,
    },
    
    #[snafu(transparent)]
    Client {
        source: client::Error,
        backtrace: Backtrace,
    }
}

pub type Result<T> = std::result::Result<T, Error>;