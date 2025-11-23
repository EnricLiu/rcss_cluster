use super::{CoachSignal, Result, Error};
use super::client;

pub struct OfflineCoach {
    conn: client::Client,
}

impl OfflineCoach {
    pub async fn send_ctrl(&mut self, ctrl: CoachSignal) -> Result<()> {
        let ctrl = ctrl.encode();
        self.conn.send(client::Signal::Data(ctrl)).await
            .map_err(|e| Error::ClientClosed { source: e })?;
        Ok(())
    }

    pub async fn shutdown(self) -> Result<()> {
        self.conn.close().await.map_err(|e| Error::ClientCloseFailed { source: e })?;
        Ok(())
    }
}