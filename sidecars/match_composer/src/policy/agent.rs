use crate::config::AgentConfig;
use crate::image::{Image, ImageProcess};
use super::Policy;

pub type AgentPolicy = Policy<AgentConfig>;

impl AgentPolicy {
    pub fn new(config: AgentConfig, image: Box<dyn Image>) -> Self {
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
            .arg("-t").arg(&self.cfg.team)
            .arg("-u").arg(self.cfg.unum.to_string())
            .arg("--g-ip").arg(self.cfg.grpc.host.to_string())
            .arg("--g-port").arg(self.cfg.grpc.port.to_string());

        if let Some(image_log_root) = &self.cfg.log_root {
            cmd.arg("--debug")
                .arg("--log-dir")
                .arg(image_log_root);
        }

        log::debug!("Spawning agent with command: {:?}", cmd);

        let stdout_log_path = self.cfg.log_root.as_ref().map(|p| {
            p.join(format!("{}_{:02}_stdout.log", &self.cfg.team, self.cfg.unum))
        });

        ImageProcess::spawn(cmd, stdout_log_path.map(|p| p.into_boxed_path()))
            .expect("Failed to spawn bot process")
    }

    pub fn unum(&self) -> u8 {
        self.cfg.unum
    }
}
