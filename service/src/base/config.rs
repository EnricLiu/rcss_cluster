use crate::base::BaseArgs;

#[derive(Clone, Debug)]
pub struct BaseConfig {
    pub half_time_auto_start: Option<u16>,
    pub always_log_stdout: bool,
}

impl From<&BaseArgs> for BaseConfig {
    fn from(args: &BaseArgs) -> Self {
        let mut ret = Self::default();
        let timesteps = args.timesteps;

        ret.half_time_auto_start = args.half_time_auto_start.then_some(timesteps / 2);
        ret.always_log_stdout = args.always_log_stdout;

        ret
    }
}

impl From<BaseArgs> for BaseConfig {
    fn from(args: BaseArgs) -> Self {
        let mut ret = Self::default();
        let timesteps = args.timesteps;

        ret.half_time_auto_start = args.half_time_auto_start.then_some(timesteps);
        ret.always_log_stdout = args.always_log_stdout;

        ret
    }
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            half_time_auto_start: None,
            always_log_stdout: true,
        }
    }
}