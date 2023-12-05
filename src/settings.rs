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
    pub save_rate: EOInt,
    pub generate_pub: bool,
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
    pub free_map: EOShort,
    pub free_x: EOChar,
    pub free_y: EOChar,
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
    pub chest_spawn_rate: EOInt,
    pub exp_multiplier: EOInt,
    pub stat_points_per_level: EOInt,
    pub skill_points_per_level: EOInt,
    pub tick_rate: EOInt,
    pub arena_spawn_rate: EOInt,
    pub door_close_rate: EOInt,
    pub quake_rate: EOInt,
    pub spike_rate: EOInt,
    pub spike_damage: f32,
    pub drain_rate: EOInt,
    pub drain_hp_damage: f32,
    pub drain_tp_damage: f32,
    pub warp_suck_rate: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct NPCs {
    pub instant_spawn: bool,
    pub respawn_rate: EOInt,
    pub freeze_on_empty_map: bool,
    pub freeze_out_of_range: bool,
    pub chase_distance: EOInt,
    pub bored_timer: EOInt,
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
pub struct Bank {
    pub max_item_amount: EOInt,
    pub base_size: EOInt,
    pub size_step: EOInt,
    pub max_upgrades: EOInt,
    pub upgrade_base_cost: EOInt,
    pub upgrade_cost_step: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Limits {
    pub max_bank_gold: EOInt,
    pub max_item: EOInt,
    pub max_trade: EOInt,
    pub max_chest: EOInt,
    pub max_party_size: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Combat {
    pub weapon_ranges: Vec<WeaponRange>,
}

#[derive(Debug, Deserialize)]
pub struct WeaponRange {
    pub weapon: EOShort,
    pub range: EOChar,
    pub arrows: bool,
}

#[derive(Debug, Deserialize)]
pub struct Board {
    pub max_posts: EOInt,
    pub max_user_posts: EOInt,
    pub max_recent_posts: EOInt,
    pub recent_post_time: EOInt,
    pub max_subject_length: EOInt,
    pub max_post_length: EOInt,
    pub date_posts: bool,
    pub admin_board: EOInt,
    pub admin_max_posts: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Chest {
    pub slots: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Quake {
    pub min_ticks: EOInt,
    pub max_ticks: EOInt,
    pub min_strength: EOInt,
    pub max_strength: EOInt,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    pub quakes: Vec<Quake>,
    pub door_close_rate: EOInt,
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
    pub bank: Bank,
    pub limits: Limits,
    pub board: Board,
    pub chest: Chest,
    pub map: Map,
    pub combat: Combat,
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
