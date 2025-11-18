use log::debug;
use crate::service::cluster::error::Error;

impl Error {

}

impl Into<super::Response> for Error {
    fn into(self) -> super::Response {
        // match self {
        //     Error::RoomNotFound { room_id } => super::Response::code_u16(500),
        //     _ =>  todo!()
        // };
        debug!("Error: {:?}", self);
        super::Response::code_u16(500)
    }
}