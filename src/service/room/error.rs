use std::backtrace::Backtrace;
use std::net::SocketAddr;
use snafu::Snafu;

use crate::service::team;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Missing field \"{field}\" to build Room."))]
    MissingField {
        field: &'static str,
        backtrace: Backtrace
    },
    #[snafu(display("Room[{room_name}]: Failed to open&connect udp socket, [{host}]->[{peer}]."))]
    UdpConn {
        room_name: String,
        host: SocketAddr,
        peer: SocketAddr,
        source: std::io::Error,
        backtrace: Backtrace
    },
    #[snafu(display("Room[{room_name}]: Udp failed to send, [{host}]->[{peer}]."))]
    UdpSend {
        room_name: String,
        host: SocketAddr,
        peer: SocketAddr,
        source: std::io::Error,
        backtrace: Backtrace
    },
    #[snafu(display("Room[{room_name}]: Udp failed to recv, [{host}]<-[{peer}]."))]
    UdpRecv {
        room_name: String,
        host: SocketAddr,
        peer: SocketAddr,
        source: std::io::Error,
        backtrace: Backtrace
    },

    #[snafu(display("Room[{room_name}]: Team[{pending_team}] failed to create on side \"{target_side}\", because Team[{occupied_team}] has occupied."))]
    RoomSideOccupied {
        room_name: String,
        pending_team: String,
        occupied_team: String,
        target_side: team::Side,
        backtrace: Backtrace,
    },

    #[snafu(display("Room[{room_name}]: Team[{pending_team}] failed to create, because the room is full."))]
    RoomIsFull {
        room_name: String,
        pending_team: String,
        backtrace: Backtrace,
    }
}

pub type Result<T> = std::result::Result<T, Error>;