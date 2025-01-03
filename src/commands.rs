use config::{Config, ConfigError, File};
use eolib::protocol::AdminLevel;

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
    pub admin_level: AdminLevel,
}

#[derive(Debug, Deserialize)]
pub struct Commands {
    pub commands: Vec<Command>,
}

impl Commands {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/Commands.ron"))
            .add_source(File::with_name("config/Commands.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }
}
