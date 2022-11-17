use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub max_connections: u32,
    pub password_salt: String,
    pub ping_rate: u32,
    pub enforce_sequence: bool,
    pub num_of_maps: u32,
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
    pub spawn_map: u32,
    pub spawn_x: u32,
    pub spawn_y: u32,
    pub spawn_direction: u32,
    pub home: String,
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    pub map: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Deserialize)]
pub struct World {
    pub see_distance: u32,
}

#[derive(Debug, Deserialize)]
pub struct NPCs {
    pub instant_spawn: bool,
    pub freeze_on_empty_map: bool,
    pub respawn_rate: u32,
    pub act_rate: u32,
    pub speed_0: u32,
    pub speed_1: u32,
    pub speed_2: u32,
    pub speed_3: u32,
    pub speed_4: u32,
    pub speed_5: u32,
    pub speed_6: u32,
    pub talk_rate: u32,
}

#[derive(Debug, Deserialize)]
pub struct SLN {
    pub enabled: bool,
    pub url: String,
    pub site: String,
    pub hostname: String,
    pub server_name: String,
    pub rate: u32,
    pub zone: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub database: Database,
    pub new_character: NewCharacter,
    pub jail: Jail,
    pub world: World,
    pub npcs: NPCs,
    pub sln: SLN,
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
