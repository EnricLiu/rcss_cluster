use log::{error, trace};
use std::io;
use std::process::Stdio;
use tokio::process::Command;

use super::*;

#[derive(Clone, Debug)]
pub struct ServerProcessSpawner {
    pgm_name: &'static str,
    pub config: Config,
}

impl ServerProcessSpawner {
    pub(super) async fn new(pgm_name: &'static str) -> Self {
        Self::validate(pgm_name).await;
        Self {
            pgm_name,
            config: Config::default_trainer_on(),
        }
    }

    async fn validate(pgm_name: &'static str) {
        // let is_exist = {
        //     let mut validate_cmd = Command::new("whereis");
        //     validate_cmd.arg(pgm_name);
        //     let out = validate_cmd.output().await
        //         .expect("error calling whereis when creating RcssServer Process");
        //
        //     let out = String::from_utf8_lossy(&out.stdout);
        //     trace!("RcssServer::validate: whereis returned: {out}");
        //     !out.ends_with(":")
        // };

        let is_exist = {
            let mut validate_cmd = Command::new("which");
            validate_cmd.arg(pgm_name);
            let out = validate_cmd
                .output()
                .await
                .expect("error calling `which` when creating RcssServer Process");

            let out = String::from_utf8_lossy(&out.stdout);
            trace!("RcssServer::validate: `which` returned: {out}");
            !out.trim().is_empty()
        };

        if !is_exist {
            error!("RcssServer::validate panic: {} is not installed", pgm_name);
            panic!("{} is not installed", pgm_name);
        }
    }

    fn build_start_cmd(&self) -> Command {
        let mut cmd = Command::new("stdbuf");
        cmd.arg("-oL").arg("-eL").arg(self.pgm_name);
        cmd.args(self.config.to_args());
        cmd
    }

    pub async fn spawn(&self) -> Result<ServerProcess> {
        let mut cmd = self.build_start_cmd();
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| match e.kind() {
            io::ErrorKind::WouldBlock => Error::MaxProcessReached(e),
            _ => Error::Io(e),
        })?;

        ServerProcess::try_from(child).await
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
