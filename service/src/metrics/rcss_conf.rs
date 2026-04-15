use serde::Serialize;
use crate::GAME_END_TIMESTEP;

#[derive(Serialize, Debug, Clone)]
pub struct RcssConfigInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trainer_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coach_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_log_dir: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_log_dir: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepaway_log_dir: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub half_time_auto_start_timestep: Option<u16>,
    pub always_log_stdout: bool,
    pub service_finish_timestep: u16,
}

impl crate::Service {
    pub async fn rcss_config_info(&self) -> RcssConfigInfo {
        let cfg = self.config();
        let base_cfg = self.base_config();

        RcssConfigInfo {
            player_port: cfg.server.port,
            trainer_port: cfg.server.coach_port,
            coach_port: cfg.server.olcoach_port,
            sync_mode: cfg.server.synch_mode,
            game_log_dir: cfg.server.game_log_dir,
            text_log_dir: cfg.server.text_log_dir,
            keepaway_log_dir: cfg.server.keepaway_log_dir,
            half_time_auto_start_timestep: base_cfg.half_time_auto_start,
            always_log_stdout: base_cfg.always_log_stdout,
            service_finish_timestep: GAME_END_TIMESTEP,
        }
    }
}
