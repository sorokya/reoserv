use config::{Config, ConfigError, File};
use eolib::protocol::net::{PacketAction, PacketFamily};

#[derive(Debug, Deserialize)]
pub struct PacketRateLimit {
    pub family: PacketFamily,
    pub action: PacketAction,
    pub limit: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct PacketRateLimits {
    #[serde(default)]
    pub packets: Vec<PacketRateLimit>,
}

impl PacketRateLimits {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/PacketRateLimits.ron"))
            .add_source(File::with_name("config/PacketRateLimits.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
