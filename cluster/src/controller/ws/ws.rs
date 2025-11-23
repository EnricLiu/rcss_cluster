use std::sync::Arc;
use uuid::Uuid;
use futures::{SinkExt, StreamExt};

use axum::{routing, Router, response::Response as AxumResponse};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use snafu::ResultExt;
use tokio::sync::mpsc;
use crate::model::signal::Signal;
use crate::service::cluster::Cluster;
use super::AppState;
use super::error::*;
use crate::ws_ensure;

async fn upgrade(
    State(s): State<AppState>,
    ws: WebSocketUpgrade, Path(client_id): Path<Uuid>
) -> AxumResponse {
    ws.on_upgrade(move |socket| async move {
        handle_upgrade(socket, s.cluster.clone(), client_id).await
    })
}

async fn handle_upgrade(socket: WebSocket, cluster: Arc<Cluster>, client_id: Uuid) -> () {
    // todo!(impl graceful shutdown using CancellationToken)
    let (mut tx, mut rx) = socket.split();
    let client = ws_ensure!(cluster.client(client_id).await, &mut tx);

    let (client_tx, mut client_rx) = mpsc::channel(32);
    let _sub_id = ws_ensure!(client.subscribe(client_tx.downgrade()).await, &mut tx);

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            #[cfg(feature = "signal-parsing")]
            let msg = {
                let signal = Signal::raw_ref(&msg);
                signal.into()
            };

            #[cfg(not(feature = "signal-parsing"))]
            let msg = Message::Text(msg.as_ref().into());

            tx.send(msg).await.context(WsSendSnafu { client_id })?;
        };
        Ok::<_, Error>(())
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = rx.next().await {
            // #[cfg(not(feature = "signal-parsing"))]
            // let msg = {
            //     let signal: Signal = msg.into();
            //     serde_json::to_string(&signal).unwrap().into_boxed_str().into()
            // };

            #[cfg(not(feature = "signal-parsing"))]
            let msg = {
                let signal = msg.into_text().expect("todo maybe no need to clone");
                signal.to_string().into_boxed_str().into()
            };

            client.send(msg).await.context(ClientSendSnafu { client_id })?;
        }
        Ok::<_, Error>(())
    });
    
    tokio::select! {
        res = &mut send_task => {
            if let Err(e) = res {
                eprintln!("WebSocket send task error for client {}: {}", client_id, e);
            }
            send_task.abort()
        },
        res = &mut recv_task => {
            if let Err(e) = res {
                eprintln!("WebSocket recv task error for client {}: {}", client_id, e);
            }
            send_task.abort()
        },
    }
}

pub fn route(path: &str) -> Router<AppState> {
    let path =
        if path == "/" { "/{client_id}" }
        else { &format!("{path}/{{client_id}}") };

    Router::new().route(path, routing::get(upgrade))
}
