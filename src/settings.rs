use config::{Config, ConfigError, File};
use eo::data::{EOChar, EOInt, EOShort};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub max_connections: EOInt,
    pub password_salt: String,
    pub ping_rate: EOInt,
    pub enforce_sequence: bool,
    pub num_of_maps: EOInt,
    pub min_version: String,
    pub max_version: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub host: String,
    pub port: String,
    pub name: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct NewCharacter {
    pub spawn_map: EOShort,
    pub spawn_x: EOChar,
    pub spawn_y: EOChar,
    pub spawn_direction: EOChar,
    pub home: String,
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    pub map: EOShort,
    pub x: EOChar,
    pub y: EOChar,
}

#[derive(Debug, Deserialize)]
pub struct Rescue {
    pub map: EOShort,
    pub x: EOChar,
    pub y: EOChar,
}

#[derive(Debug, Deserialize)]
pub struct World {
    pub see_distance: EOChar,
    pub drop_distance: EOChar,
    pub recover_rate: EOInt,
    pub npc_recover_rate: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct NPCs {
    pub instant_spawn: bool,
    pub freeze_on_empty_map: bool,
    pub chase_distance: EOInt,
    pub bored_timer: EOInt,
    pub respawn_rate: EOInt,
    pub act_rate: EOInt,
    pub speed_0: EOInt,
    pub speed_1: EOInt,
    pub speed_2: EOInt,
    pub speed_3: EOInt,
    pub speed_4: EOInt,
    pub speed_5: EOInt,
    pub speed_6: EOInt,
    pub talk_rate: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Sln {
    pub enabled: bool,
    pub url: String,
    pub site: String,
    pub hostname: String,
    pub server_name: String,
    pub rate: EOInt,
    pub zone: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub new_character: NewCharacter,
    pub jail: Jail,
    pub rescue: Rescue,
    pub world: World,
    pub npcs: NPCs,
    pub sln: Sln,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("Config.toml"))
            .add_source(File::with_name("Config.local.toml").required(false))
            .build()?;

        s.try_deserialize()
    }
}
