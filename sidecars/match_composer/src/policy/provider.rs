use std::fmt::Debug;
use std::path::Path;
use std::process::Command;

use crate::config::{ImageConfig, PlayerConfig};

pub struct Image {
    pub cfg: ImageConfig,
    pub path: Box<Path>,
}

impl Image {
    pub fn new<E: Debug>(config: impl TryInto<ImageConfig, Error=E>, path: impl AsRef<Path>) -> Self {
        Image {
            cfg: config.try_into().expect("Invalid image config"),
            path: path.as_ref().into(),
        }
    }

    pub fn create_player_cmd(
        &self,config: &PlayerConfig,
    ) -> Command {
        let mut cmd = Command::new(self.path.join("start.sh"));
        cmd.arg("-h")
            .arg(config.host.to_string())
            .arg("-p")
            .arg(config.port.to_string())
            .arg("-t")
            .arg(config.team_name)
            .arg("-u")
            .arg(config.unum.to_string())
            .arg("--log-dir")
            .arg(config.log_path);

        cmd
    }
}

pub struct ImageRegistry {
    pub local: Box<Path>,
}

impl ImageRegistry {
    pub fn new(local: impl AsRef<Path>) -> ImageRegistry {
        ImageRegistry {
            local: local.as_ref().into(),
        }
    }

    pub fn models(&self, provider: &str) -> Option<impl Iterator<Item=Image>> {
        let dir = match self.local.join(provider).read_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let ret = dir.filter_map(|entry| {
            entry.ok().and_then(|ent| {
                if  let Ok(ty) = ent.file_type() && ty.is_file() &&
                    let Ok(model) = ent.file_name().into_string() {
                    return Some(Image::new(ImageConfig {
                        provider: provider.to_string(),
                        model,
                    }, ent.path()))
                }
                None
            })
        });

        Some(ret)
    }

    pub fn providers(&self) -> Option<impl Iterator<Item=String>> {
        let dir = match self.local.read_dir() {
            Ok(dir) => dir,
            Err(_) => return None,
        };

        let ret = dir.filter_map(|entry| {
            entry.ok().and_then(|ent| {
                if  let Ok(ty) = ent.file_type() && ty.is_dir() &&
                    let Ok(provider) = ent.file_name().into_string() {
                    return Some(provider)
                }
                None
            })
        });

        Some(ret)
    }
    
    pub fn try_get(&self, provider: &str, model: &str) -> Option<Image> {
        let dir = self.local.join(provider).join(model);
        dir.is_dir().then_some(
            Image::new(ImageConfig {
                provider: provider.to_string(),
                model: model.to_string(),
            }, dir)
        )
    }
}