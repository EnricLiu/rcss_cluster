use std::path::PathBuf;

use crate::image::{Image, ImageProcess};
use crate::config::AgentConfig;
use super::Policy;

pub type AgentPolicy<'a> = Policy<AgentConfig<'a>>;


impl<'a> AgentPolicy<'a> {
    pub fn new(config: AgentConfig<'a>, image: Box<dyn Image>) -> Self {
        AgentPolicy {
            cfg: config,
            image,
        }
    }

    pub async fn spawn(&self) -> ImageProcess {
        let mut cmd = self.image.cmd();
        cmd
            .arg("-h").arg(self.cfg.server.host.to_string())
            .arg("-p").arg(self.cfg.server.port.to_string())
            .arg("-t").arg(self.cfg.team)
            .arg("-u").arg(self.cfg.unum.to_string())
            .arg("--g-ip").arg(self.cfg.grpc.host.to_string())
            .arg("--g-port").arg(self.cfg.grpc.port.to_string());

        if let Some(log_dir) = &self.cfg.log_path {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(log_dir);
        }

        ImageProcess::spawn(cmd, Some(PathBuf::from("./logs/test.log").into_boxed_path()))
            .expect("Failed to spawn bot process")
    }

    pub fn unum(&self) -> u8 {
        self.cfg.unum
    }
}
