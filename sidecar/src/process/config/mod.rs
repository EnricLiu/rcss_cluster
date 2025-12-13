mod config;
pub mod csv_saver;
pub mod player;
pub mod server;

pub use csv_saver::CsvSaverConfig;
pub use player::PlayerConfig;
pub use server::ServerConfig;

pub use config::Config;

#[macro_export]
macro_rules! create_config {
    ($ident:ident, $namespace:literal, {$($field:ident: $value:ty),+$(,)?}) => {
        #[derive(Clone, Debug, Default)]
        pub struct $ident {
            $(
                pub $field: Option<$value>,
            )*
        }

        impl $ident {
            $(
                pub fn $field(&mut self, value: $value) -> &mut Self {
                    self.$field = Some(value);
                    self
                }
            )*
        }

        impl $ident {
            pub fn to_args(&self) -> Vec<String> {
                let mut args = vec![];
                $(
                    if let Some(value) = &self.$field {
                        args.push(format!("{}::{}={value}", $namespace, stringify!($field)));
                    }
                )*
                args
            }
        }
    }
}
