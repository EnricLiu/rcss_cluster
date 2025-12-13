mod trainer;

use axum::Router;
use serde::Serialize;

use super::{AppState, Error, Response};

use common::command::{Command, CommandResult};

#[derive(Debug)]
pub struct CommandResponse<C: Command>(CommandResult<C>);

#[derive(Serialize, Debug)]
struct SerializeCommandResponseHelper<'a, C: Command> {
    ok: bool,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    ok_content: Option<&'a C::Ok>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    err: Option<&'a String>,
}

impl<C: Command> Serialize for CommandResponse<C> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let helper: SerializeCommandResponseHelper<'_, C> = match &self.0 {
            Ok(ok) => SerializeCommandResponseHelper {
                ok: true,
                ok_content: Some(ok),
                err: None,
            },
            Err(err) => SerializeCommandResponseHelper {
                ok: false,
                ok_content: None,
                err: Some(&err.to_string()),
            },
        };

        helper.serialize(serializer)
    }
}

impl<C: Command> From<CommandResult<C>> for CommandResponse<C> {
    fn from(result: CommandResult<C>) -> Self {
        CommandResponse(result)
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let inner = Router::new().merge(trainer::route("/trainer"));

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
