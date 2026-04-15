use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct AgonesMcConfigInfo {
	pub port: u16,
	pub addr: String,
	pub status_poll_interval_ms: u64,
	pub connect_timeout_ms: u64,
	pub request_timeout_ms: u64,
	pub start_max_retries: u32,
	pub start_retry_base_ms: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct AgonesConfigInfo {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sdk_port: Option<u16>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sdk_keep_alive_ms: Option<u64>,
	pub health_check_interval_ms: u64,
	pub auto_shutdown_on_finish: bool,
	
	pub match_composer_enabled: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub match_composer: Option<AgonesMcConfigInfo>,
}

#[inline]
fn duration_ms(duration: std::time::Duration) -> u64 {
	duration.as_millis().try_into().unwrap_or(u64::MAX)
}

impl crate::Service {
	pub async fn agones_info(&self) -> AgonesConfigInfo {
		let cfg = self.agones_config();
		let match_composer = cfg.match_composer.as_ref().map(|mc| AgonesMcConfigInfo {
			port: mc.port,
			addr: mc.client_cfg.addr.to_string(),
			status_poll_interval_ms: duration_ms(mc.status_poll_interval),
			connect_timeout_ms: duration_ms(mc.client_cfg.connect_timeout),
			request_timeout_ms: duration_ms(mc.client_cfg.request_timeout),
			start_max_retries: mc.client_cfg.start_max_retries,
			start_retry_base_ms: duration_ms(mc.client_cfg.start_retry_base),
		});

		AgonesConfigInfo {
			health_check_interval_ms: duration_ms(cfg.health_check_interval),
			auto_shutdown_on_finish: cfg.shutdown.on_finish,
			sdk_port: cfg.sdk.port,
			sdk_keep_alive_ms: cfg.sdk.keep_alive.map(duration_ms),
			match_composer_enabled: match_composer.is_some(),
			match_composer,
		}
	}
}

