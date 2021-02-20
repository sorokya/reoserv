use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("Config.toml"))?;
        s.merge(File::with_name("Config.local.toml").required(false))?;
        s.try_into()
    }
}
