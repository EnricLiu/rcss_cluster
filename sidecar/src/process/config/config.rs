use super::{CsvSaverConfig, PlayerConfig, ServerConfig};

pub const LOG_DIR: &str = "./log";

#[derive(Clone, Debug)]
pub struct Config {
    pub server: ServerConfig,
    pub player: PlayerConfig,
    pub csv_saver: CsvSaverConfig,
}

impl Default for Config {
    fn default() -> Self {
        let mut ret = Self {
            server: ServerConfig::default(),
            player: PlayerConfig::default(),
            csv_saver: CsvSaverConfig::default(),
        };

        ret.with_all_log_dir(LOG_DIR);

        ret
    }
}

impl Config {
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![];
        args.append(&mut self.server.to_args());
        args.append(&mut self.player.to_args());
        args.append(&mut self.csv_saver.to_args());
        args
    }

    pub fn default_trainer_on() -> Self {
        let mut ret = Self::default();
        ret.server_then(|cfg| {
            cfg.coach(true).coach_w_referee(true).synch_mode(true);
        });
        ret
    }

    pub fn with_ports(&mut self, port: u16, coach_port: u16, olcoach_port: u16) -> &mut Self {
        self.server_then(|c| {
            c.port(port)
                .coach_port(coach_port)
                .olcoach_port(olcoach_port);
        })
    }

    pub fn with_sync(&mut self, sync: bool) -> &mut Self {
        self.server_then(|c| {
            c.synch_mode(sync);
        })
    }

    pub fn with_log_dir(&mut self, log_dir: &'static str) -> &mut Self {
        self.server_then(|c| {
            c.game_log_dir(log_dir);
        })
    }

    pub fn with_all_log_dir(&mut self, log_dir: &'static str) -> &mut Self {
        self.server_then(|c| {
            c.game_log_dir(log_dir);
            c.text_log_dir(log_dir);
            c.keepaway_log_dir(log_dir);
        })
    }

    #[inline]
    pub fn with_server(&mut self, server: ServerConfig) -> &mut Self {
        self.server = server;
        self
    }
    #[inline]
    pub fn with_player(&mut self, player: PlayerConfig) -> &mut Self {
        self.player = player;
        self
    }
    #[inline]
    pub fn with_csv_saver(&mut self, csv_saver: CsvSaverConfig) -> &mut Self {
        self.csv_saver = csv_saver;
        self
    }

    #[inline]
    pub fn server_then(&mut self, f: impl FnOnce(&mut ServerConfig)) -> &mut Self {
        f(&mut self.server);
        self
    }
    #[inline]
    pub fn player_then(&mut self, f: impl FnOnce(&mut PlayerConfig)) -> &mut Self {
        f(&mut self.player);
        self
    }
    #[inline]
    pub fn csv_saver_then(&mut self, f: impl FnOnce(&mut CsvSaverConfig)) -> &mut Self {
        f(&mut self.csv_saver);
        self
    }
}
