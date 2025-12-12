use std::net::SocketAddr;
use std::sync::Arc;
use crate::room::{Room, RoomConfig};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Room[{room_name}] not found")]
    RoomNotFound {
        room_name: String,
    },
    
    #[error("Room[{room_name}] already dropped")]
    RoomDropped {
        room_name: String,
    },

    #[error("Room[{room_name}] was locked and can not drop, insert it back to the room.")]
    RoomDropRetrieved {
        room_name: String,
    },

    #[error("Room[{room_name}] was locked and can not drop or insert back.")]
    RoomDropDangled {
        room_name: String,
        room: Arc<Room>,
    },
    
    #[error(transparent)]
    Room(#[from] crate::room::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
