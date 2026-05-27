use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub enum ArgType {
    UInt,
    String,
    Bool,
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub r#type: ArgType,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    pub usage: String,
    pub alias: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerCommands {
    #[serde(default)]
    pub commands: Vec<Command>,
}

impl Default for PlayerCommands {
    fn default() -> Self {
        Self { commands: vec![] }
    }
}

impl PlayerCommands {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/PlayerCommands.ron"))
            .add_source(File::with_name("config/PlayerCommands.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
