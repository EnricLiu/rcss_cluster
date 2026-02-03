use log::debug;
use common::command::player::CommandInit;
use crate::client::RichClientBuilder;
use super::Player;

#[derive(Clone, Debug)]
pub struct PlayerBuilder {
    pub rich_client: RichClientBuilder,
    pub enable_resolver: bool,
    pub init_on_build: Option<CommandInit>,
}

impl PlayerBuilder {
    pub fn new() -> Self {
        let rich_client = RichClientBuilder::player();
        let enable_resolver = false;
        let init_on_build = None;

        Self {
            rich_client,
            enable_resolver,
            init_on_build,
        }
    }

    pub fn enable_resolver(&mut self) -> &mut Self {
        self.enable_resolver = true;
        self
    }
    
    pub fn disable_resolver(&mut self) -> &mut Self {
        self.enable_resolver = false;
        self
    }
    
    pub fn init_on_build(&mut self, init_on_build: CommandInit) -> &mut Self {
        self.with_init_on_build(Some(init_on_build))
    }
    
    pub fn with_init_on_build(&mut self, init_on_build: Option<CommandInit>) -> &mut Self {
        self.init_on_build = init_on_build;
        self
    }

    pub async fn build(&self) -> Result<Player, String> {
        todo!()
    }

    pub async fn build_into(self) -> Result<Player, String> {
        let client = self.rich_client.build_into();
        let player = Player {
            client,
        };
        if !self.enable_resolver {
            return Ok(player)
        }
        
        player.init_resolver().map_err(|err| err.to_string())?;
        debug!("[Player] CallResolver initialized.");
        
        if let Some(init_msg) = self.init_on_build {
            player.connect().await.map_err(|err| err.to_string())?;
            let res = player.call(init_msg).await.map_err(|err| err.to_string())?;
            if let Err(err_init) = res {
                return Err(err_init.to_string())
            }
        }
        Ok(player)
    }
}