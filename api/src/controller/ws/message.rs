use crate::model::signal::Signal;
use axum::extract::ws::Message;

impl<'a> Into<Message> for Signal<'a> {
    fn into(self) -> Message {
        let str = serde_json::to_string(&self).expect("Failed to serialize Signal");
        Message::Text(str.into())
    }
}

impl Into<Signal<'static>> for Message {
    fn into(self) -> Signal<'static> {
        match self {
            Message::Text(text) => {
                let msg: Signal<'_> =
                    serde_json::from_str(&text).expect("Failed to deserialize Message");
                msg.into_owned()
            }
            _ => todo!("Handle other message types"),
        }
    }
}
