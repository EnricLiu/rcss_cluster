use crate::model::signal::Signal;
use axum::extract::ws::Message;

impl<'a> From<Signal<'a>> for Message {
    fn from(val: Signal<'a>) -> Self {
        let str = serde_json::to_string(&val).expect("Failed to serialize Signal");
        Message::Text(str.into())
    }
}

impl From<Message> for Signal<'static> {
    fn from(val: Message) -> Self {
        match val {
            Message::Text(text) => {
                let msg: Signal<'_> =
                    serde_json::from_str(&text).expect("Failed to deserialize Message");
                msg.into_owned()
            }
            _ => todo!("Handle other message types"),
        }
    }
}
