mod error;
mod player;

use super::AppState;
use axum::Router;

#[macro_export]
macro_rules! ws_ensure {
    ($result:expr, $sender:expr) => {{
        let sender = $sender;
        match $result {
            Ok(v) => v,
            Err(e) => {
                let _ = sender
                    .send($crate::model::signal::Signal::error(&e).into())
                    .await;
                let _ = sender.close();
                return;
            }
        }
    }};
}

pub fn route(path: &str, app_state: AppState) -> Router {
    let inner = Router::new()
        .merge(player::route("/"))
        .with_state(app_state);

    if path == "/" {
        inner
    } else {
        Router::new().nest(path, inner)
    }
}
