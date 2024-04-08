use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Arena {
    pub map: i32,
    pub rate: i32,
    pub block: i32,
    pub spawns: Vec<ArenaSpawn>,
}

#[derive(Debug, Deserialize)]
pub struct ArenaSpawn {
    pub from: ArenaCoords,
    pub to: ArenaCoords,
}

#[derive(Debug, Deserialize)]
pub struct ArenaCoords {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize)]
pub struct Arenas {
    pub arenas: Vec<Arena>,
}

impl Arenas {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/Arenas.ron"))
            .add_source(File::with_name("config/Arenas.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }
}
