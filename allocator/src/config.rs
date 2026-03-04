use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Scheduling {
    Packed,
    Distributed,
}

impl Scheduling {
    pub fn as_str(&self) -> &'static str {
        match self {
            Scheduling::Packed => "Packed",
            Scheduling::Distributed => "Distributed",
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(name = "allocator")]
#[command(about = "Custom Allocator for Agones GameServer allocation")]
pub struct Config {
    /// HTTP server bind address
    #[arg(long, env = "BIND_ADDRESS", default_value = "0.0.0.0:8080")]
    pub bind_address: String,

    /// Target Agones Fleet name
    #[arg(long, env = "FLEET_NAME", default_value = "rl-training-fleet")]
    pub fleet_name: String,

    /// Kubernetes namespace
    #[arg(long, env = "NAMESPACE", default_value = "default")]
    pub namespace: String,

    /// Maximum allowed bot count
    #[arg(long, env = "MAX_BOT_COUNT", default_value = "20")]
    pub max_bot_count: u32,

    /// Minimum allowed bot count
    #[arg(long, env = "MIN_BOT_COUNT", default_value = "1")]
    pub min_bot_count: u32,

    /// Bearer token for authentication (optional)
    #[arg(long, env = "AUTH_TOKEN")]
    pub auth_token: Option<String>,

    /// Allowed client versions (comma-separated, empty means all versions allowed)
    #[arg(long, env = "ALLOWED_VERSIONS", value_delimiter = ',')]
    pub allowed_versions: Vec<String>,

    /// Scheduling strategy for GameServer allocation
    #[arg(long, env = "SCHEDULING", default_value = "packed")]
    pub scheduling: Scheduling,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_bot_count == 0 {
            return Err("min_bot_count must be greater than 0".to_string());
        }
        if self.max_bot_count < self.min_bot_count {
            return Err("max_bot_count must be >= min_bot_count".to_string());
        }
        Ok(())
    }

    pub fn is_bot_count_valid(&self, count: u32) -> bool {
        count >= self.min_bot_count && count <= self.max_bot_count
    }

    pub fn is_version_allowed(&self, version: &str) -> bool {
        if self.allowed_versions.is_empty() {
            return true;
        }
        self.allowed_versions.iter().any(|v| v == version)
    }
}
