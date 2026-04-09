use std::path::PathBuf;
use super::RcssServerConfig;

#[derive(Clone, Debug)]
pub struct MatchComposerConfig {
    pub server: RcssServerConfig,
    pub log_root: Option<PathBuf>,
    pub registry_path: PathBuf,
}
