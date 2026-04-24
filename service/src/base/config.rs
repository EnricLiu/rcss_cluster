use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use crate::base::BaseArgs;

#[derive(Clone, Debug)]
pub struct BaseConfig {
    pub half_time_auto_start: Option<u16>,
    pub always_log_stdout: bool,
    pub log_root: OnceLock<PathBuf>,
    pub rcss_game_log_rel_dir: PathBuf,
    pub rcss_stdio_log_rel_path: Option<PathBuf>,
}

impl BaseConfig {
    pub fn set_log_root(&self, path: PathBuf) -> &Path {
        self.log_root.get_or_init(||path)
    }

    pub fn log_root(&self) -> &Path {
        self.log_root.get().unwrap()
    }
}

impl From<&BaseArgs> for BaseConfig {
    fn from(args: &BaseArgs) -> Self {
        let mut ret = Self::default();
        let timesteps = args.rcss_max_timesteps;

        ret.half_time_auto_start = args.half_time_auto_start.then_some(timesteps / 2);
        ret.always_log_stdout = args.always_log_stdout;
        ret.rcss_game_log_rel_dir = args.rcss_game_log_dir.clone();

        ret
    }
}

impl From<BaseArgs> for BaseConfig {
    fn from(args: BaseArgs) -> Self {
        Self::from(&args)
    }
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            half_time_auto_start: None,
            always_log_stdout: true,
            log_root: OnceLock::new(),
            rcss_game_log_rel_dir: PathBuf::from("./games"),
            rcss_stdio_log_rel_path: None,
        }
    }
}