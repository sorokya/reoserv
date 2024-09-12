use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub enum ArgType {
    UInt,
    String,
    Bool,
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub name: String,
    pub r#type: ArgType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub min: i32,
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub alias: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerCommands {
    pub commands: Vec<Command>,
}

impl PlayerCommands {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/PlayerCommands.ron"))
            .add_source(File::with_name("config/PlayerCommands.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }
}
