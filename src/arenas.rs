use config::{Config, ConfigError, File};
use eo::data::{EOChar, EOInt, EOShort};

#[derive(Debug, Deserialize)]
pub struct Arena {
    pub map: EOShort,
    pub rate: EOInt,
    pub block: EOChar,
    pub spawns: Vec<ArenaSpawn>,
}

#[derive(Debug, Deserialize)]
pub struct ArenaSpawn {
    pub from: ArenaCoords,
    pub to: ArenaCoords,
}

#[derive(Debug, Deserialize)]
pub struct ArenaCoords {
    pub x: EOChar,
    pub y: EOChar,
}

#[derive(Debug, Deserialize)]
pub struct Arenas {
    pub arenas: Vec<Arena>,
}

impl Arenas {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("Arenas.ron"))
            .add_source(File::with_name("Arenas.local.ron").required(false))
            .build()?;

        s.try_deserialize()
    }
}
