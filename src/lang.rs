use config::{Config, ConfigError, File};

use crate::SETTINGS;

#[derive(Debug, Deserialize)]
pub struct Lang {
    pub announce_freeze: String,
    pub announce_unfreeze: String,
    pub announce_remove: String,
    pub announce_mute: String,
    pub announce_global: String,
    pub global_locked: String,
}

impl Lang {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name(&format!(
                "lang/{}.ron",
                SETTINGS.server.lang
            )))
            .build()?;

        s.try_deserialize()
    }
}
