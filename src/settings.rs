use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Server {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: String,
    #[serde(default = "default_server_websocket_port")]
    pub websocket_port: String,
    #[serde(default = "default_server_max_connections")]
    pub max_connections: i32,
    #[serde(default = "default_server_max_players")]
    pub max_players: i32,
    #[serde(default = "default_server_max_connections_per_ip")]
    pub max_connections_per_ip: i32,
    #[serde(default = "default_server_ip_reconnect_limit")]
    pub ip_reconnect_limit: i32,
    #[serde(default = "default_server_hangup_delay")]
    pub hangup_delay: i32,
    #[serde(default = "default_server_max_login_attempts")]
    pub max_login_attempts: i32,
    #[serde(default = "default_server_ping_rate")]
    pub ping_rate: i32,
    #[serde(default = "default_server_enforce_sequence")]
    pub enforce_sequence: bool,
    #[serde(default = "default_server_min_version")]
    pub min_version: String,
    #[serde(default = "default_server_max_version")]
    pub max_version: String,
    #[serde(default = "default_server_save_rate")]
    pub save_rate: i32,
    #[serde(default = "default_server_generate_pub")]
    pub generate_pub: bool,
    #[serde(default = "default_server_lang")]
    pub lang: String,
    #[serde(default = "default_server_auto_admin")]
    pub auto_admin: bool,
}

fn default_server_host() -> String { "0.0.0.0".to_string() }
fn default_server_port() -> String { "8078".to_string() }
fn default_server_websocket_port() -> String { "".to_string() }
fn default_server_max_connections() -> i32 { 300 }
fn default_server_max_players() -> i32 { 200 }
fn default_server_max_connections_per_ip() -> i32 { 3 }
fn default_server_ip_reconnect_limit() -> i32 { 10 }
fn default_server_hangup_delay() -> i32 { 10 }
fn default_server_max_login_attempts() -> i32 { 3 }
fn default_server_ping_rate() -> i32 { 60 }
fn default_server_enforce_sequence() -> bool { true }
fn default_server_min_version() -> String { "0.0.28".to_string() }
fn default_server_max_version() -> String { "0.3.29".to_string() }
fn default_server_save_rate() -> i32 { 5 }
fn default_server_generate_pub() -> bool { false }
fn default_server_lang() -> String { "en".to_string() }
fn default_server_auto_admin() -> bool { true }

impl Default for Server {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
            websocket_port: default_server_websocket_port(),
            max_connections: default_server_max_connections(),
            max_players: default_server_max_players(),
            max_connections_per_ip: default_server_max_connections_per_ip(),
            ip_reconnect_limit: default_server_ip_reconnect_limit(),
            hangup_delay: default_server_hangup_delay(),
            max_login_attempts: default_server_max_login_attempts(),
            ping_rate: default_server_ping_rate(),
            enforce_sequence: default_server_enforce_sequence(),
            min_version: default_server_min_version(),
            max_version: default_server_max_version(),
            save_rate: default_server_save_rate(),
            generate_pub: default_server_generate_pub(),
            lang: default_server_lang(),
            auto_admin: default_server_auto_admin(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Database {
    #[serde(default = "default_database_driver")]
    pub driver: String,
    #[serde(default = "default_database_host")]
    pub host: String,
    #[serde(default = "default_database_port")]
    pub port: String,
    #[serde(default = "default_database_name")]
    pub name: String,
    #[serde(default = "default_database_username")]
    pub username: String,
    #[serde(default = "default_database_password")]
    pub password: String,
}

fn default_database_driver() -> String { "sqlite".to_string() }
fn default_database_host() -> String { "127.0.0.1".to_string() }
fn default_database_port() -> String { "3306".to_string() }
fn default_database_name() -> String { "reoserv".to_string() }
fn default_database_username() -> String { "reoserv".to_string() }
fn default_database_password() -> String { "reoserv".to_string() }

impl Default for Database {
    fn default() -> Self {
        Self {
            driver: default_database_driver(),
            host: default_database_host(),
            port: default_database_port(),
            name: default_database_name(),
            username: default_database_username(),
            password: default_database_password(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewCharacter {
    #[serde(default = "default_new_character_spawn_map")]
    pub spawn_map: i32,
    #[serde(default = "default_new_character_spawn_x")]
    pub spawn_x: i32,
    #[serde(default = "default_new_character_spawn_y")]
    pub spawn_y: i32,
    #[serde(default = "default_new_character_spawn_direction")]
    pub spawn_direction: i32,
    #[serde(default = "default_new_character_home")]
    pub home: String,
}

fn default_new_character_spawn_map() -> i32 { 192 }
fn default_new_character_spawn_x() -> i32 { 6 }
fn default_new_character_spawn_y() -> i32 { 6 }
fn default_new_character_spawn_direction() -> i32 { 0 }
fn default_new_character_home() -> String { "Wanderer".to_string() }

impl Default for NewCharacter {
    fn default() -> Self {
        Self {
            spawn_map: default_new_character_spawn_map(),
            spawn_x: default_new_character_spawn_x(),
            spawn_y: default_new_character_spawn_y(),
            spawn_direction: default_new_character_spawn_direction(),
            home: default_new_character_home(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Character {
    #[serde(default = "default_character_min_name_length")]
    pub min_name_length: usize,
    #[serde(default = "default_character_max_name_length")]
    pub max_name_length: usize,
    #[serde(default = "default_character_max_title_length")]
    pub max_title_length: usize,
    #[serde(default = "default_character_max_skin")]
    pub max_skin: i32,
    #[serde(default = "default_character_max_hair_color")]
    pub max_hair_color: i32,
    #[serde(default = "default_character_max_hair_style")]
    pub max_hair_style: i32,
}

fn default_character_min_name_length() -> usize { 3 }
fn default_character_max_name_length() -> usize { 12 }
fn default_character_max_title_length() -> usize { 32 }
fn default_character_max_skin() -> i32 { 11 }
fn default_character_max_hair_color() -> i32 { 9 }
fn default_character_max_hair_style() -> i32 { 20 }

impl Default for Character {
    fn default() -> Self {
        Self {
            min_name_length: default_character_min_name_length(),
            max_name_length: default_character_max_name_length(),
            max_title_length: default_character_max_title_length(),
            max_skin: default_character_max_skin(),
            max_hair_color: default_character_max_hair_color(),
            max_hair_style: default_character_max_hair_style(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    #[serde(default = "default_jail_map")]
    pub map: i32,
    #[serde(default = "default_jail_x")]
    pub x: i32,
    #[serde(default = "default_jail_y")]
    pub y: i32,
    #[serde(default = "default_jail_free_map")]
    pub free_map: i32,
    #[serde(default = "default_jail_free_x")]
    pub free_x: i32,
    #[serde(default = "default_jail_free_y")]
    pub free_y: i32,
}

fn default_jail_map() -> i32 { 76 }
fn default_jail_x() -> i32 { 6 }
fn default_jail_y() -> i32 { 7 }
fn default_jail_free_map() -> i32 { 76 }
fn default_jail_free_x() -> i32 { 9 }
fn default_jail_free_y() -> i32 { 11 }

impl Default for Jail {
    fn default() -> Self {
        Self {
            map: default_jail_map(),
            x: default_jail_x(),
            y: default_jail_y(),
            free_map: default_jail_free_map(),
            free_x: default_jail_free_x(),
            free_y: default_jail_free_y(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Rescue {
    #[serde(default = "default_rescue_map")]
    pub map: i32,
    #[serde(default = "default_rescue_x")]
    pub x: i32,
    #[serde(default = "default_rescue_y")]
    pub y: i32,
}

fn default_rescue_map() -> i32 { 4 }
fn default_rescue_x() -> i32 { 24 }
fn default_rescue_y() -> i32 { 24 }

impl Default for Rescue {
    fn default() -> Self {
        Self {
            map: default_rescue_map(),
            x: default_rescue_x(),
            y: default_rescue_y(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct World {
    #[serde(default = "default_world_drop_distance")]
    pub drop_distance: i32,
    #[serde(default = "default_world_drop_protect_player")]
    pub drop_protect_player: i32,
    #[serde(default = "default_world_drop_protect_npc")]
    pub drop_protect_npc: i32,
    #[serde(default = "default_world_recover_rate")]
    pub recover_rate: i32,
    #[serde(default = "default_world_npc_recover_rate")]
    pub npc_recover_rate: i32,
    #[serde(default = "default_world_chest_spawn_rate")]
    pub chest_spawn_rate: i32,
    #[serde(default = "default_world_exp_multiplier")]
    pub exp_multiplier: i32,
    #[serde(default = "default_world_stat_points_per_level")]
    pub stat_points_per_level: i32,
    #[serde(default = "default_world_skill_points_per_level")]
    pub skill_points_per_level: i32,
    #[serde(default = "default_world_tick_rate")]
    pub tick_rate: i32,
    #[serde(default = "default_world_quake_rate")]
    pub quake_rate: i32,
    #[serde(default = "default_world_spike_rate")]
    pub spike_rate: i32,
    #[serde(default = "default_world_spike_damage")]
    pub spike_damage: f32,
    #[serde(default = "default_world_drain_rate")]
    pub drain_rate: i32,
    #[serde(default = "default_world_drain_hp_damage")]
    pub drain_hp_damage: f32,
    #[serde(default = "default_world_drain_tp_damage")]
    pub drain_tp_damage: f32,
    #[serde(default = "default_world_warp_suck_rate")]
    pub warp_suck_rate: i32,
    #[serde(default = "default_world_ghost_rate")]
    pub ghost_rate: i32,
    #[serde(default = "default_world_info_reveals_drops")]
    pub info_reveals_drops: bool,
    #[serde(default = "default_world_usage_rate")]
    pub usage_rate: i32,
}

fn default_world_drop_distance() -> i32 { 2 }
fn default_world_drop_protect_player() -> i32 { 5 }
fn default_world_drop_protect_npc() -> i32 { 30 }
fn default_world_recover_rate() -> i32 { 720 }
fn default_world_npc_recover_rate() -> i32 { 840 }
fn default_world_chest_spawn_rate() -> i32 { 480 }
fn default_world_exp_multiplier() -> i32 { 1 }
fn default_world_stat_points_per_level() -> i32 { 3 }
fn default_world_skill_points_per_level() -> i32 { 4 }
fn default_world_tick_rate() -> i32 { 125 }
fn default_world_quake_rate() -> i32 { 25 }
fn default_world_spike_rate() -> i32 { 12 }
fn default_world_spike_damage() -> f32 { 0.2 }
fn default_world_drain_rate() -> i32 { 125 }
fn default_world_drain_hp_damage() -> f32 { 0.1 }
fn default_world_drain_tp_damage() -> f32 { 0.1 }
fn default_world_warp_suck_rate() -> i32 { 15 }
fn default_world_ghost_rate() -> i32 { 5 }
fn default_world_info_reveals_drops() -> bool { true }
fn default_world_usage_rate() -> i32 { 60 }

impl Default for World {
    fn default() -> Self {
        Self {
            drop_distance: default_world_drop_distance(),
            drop_protect_player: default_world_drop_protect_player(),
            drop_protect_npc: default_world_drop_protect_npc(),
            recover_rate: default_world_recover_rate(),
            npc_recover_rate: default_world_npc_recover_rate(),
            chest_spawn_rate: default_world_chest_spawn_rate(),
            exp_multiplier: default_world_exp_multiplier(),
            stat_points_per_level: default_world_stat_points_per_level(),
            skill_points_per_level: default_world_skill_points_per_level(),
            tick_rate: default_world_tick_rate(),
            quake_rate: default_world_quake_rate(),
            spike_rate: default_world_spike_rate(),
            spike_damage: default_world_spike_damage(),
            drain_rate: default_world_drain_rate(),
            drain_hp_damage: default_world_drain_hp_damage(),
            drain_tp_damage: default_world_drain_tp_damage(),
            warp_suck_rate: default_world_warp_suck_rate(),
            ghost_rate: default_world_ghost_rate(),
            info_reveals_drops: default_world_info_reveals_drops(),
            usage_rate: default_world_usage_rate(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Bard {
    #[serde(default = "default_bard_instrument_items")]
    pub instrument_items: Vec<i32>,
    #[serde(default = "default_bard_max_note_id")]
    pub max_note_id: i32,
}

fn default_bard_instrument_items() -> Vec<i32> { vec![49, 50] }
fn default_bard_max_note_id() -> i32 { 36 }

impl Default for Bard {
    fn default() -> Self {
        Self {
            instrument_items: default_bard_instrument_items(),
            max_note_id: default_bard_max_note_id(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NPCs {
    #[serde(default = "default_npcs_instant_spawn")]
    pub instant_spawn: bool,
    #[serde(default = "default_npcs_freeze_on_empty_map")]
    pub freeze_on_empty_map: bool,
    #[serde(default = "default_npcs_chase_distance")]
    pub chase_distance: i32,
    #[serde(default = "default_npcs_bored_timer")]
    pub bored_timer: i32,
    #[serde(default = "default_npcs_act_rate")]
    pub act_rate: i32,
    #[serde(default = "default_npcs_speed_0")]
    pub speed_0: i32,
    #[serde(default = "default_npcs_speed_1")]
    pub speed_1: i32,
    #[serde(default = "default_npcs_speed_2")]
    pub speed_2: i32,
    #[serde(default = "default_npcs_speed_3")]
    pub speed_3: i32,
    #[serde(default = "default_npcs_speed_4")]
    pub speed_4: i32,
    #[serde(default = "default_npcs_speed_5")]
    pub speed_5: i32,
    #[serde(default = "default_npcs_speed_6")]
    pub speed_6: i32,
    #[serde(default = "default_npcs_talk_rate")]
    pub talk_rate: i32,
}

fn default_npcs_instant_spawn() -> bool { false }
fn default_npcs_freeze_on_empty_map() -> bool { true }
fn default_npcs_chase_distance() -> i32 { 10 }
fn default_npcs_bored_timer() -> i32 { 240 }
fn default_npcs_act_rate() -> i32 { 5 }
fn default_npcs_speed_0() -> i32 { 5 }
fn default_npcs_speed_1() -> i32 { 5 }
fn default_npcs_speed_2() -> i32 { 10 }
fn default_npcs_speed_3() -> i32 { 15 }
fn default_npcs_speed_4() -> i32 { 30 }
fn default_npcs_speed_5() -> i32 { 60 }
fn default_npcs_speed_6() -> i32 { 120 }
fn default_npcs_talk_rate() -> i32 { 300 }

impl Default for NPCs {
    fn default() -> Self {
        Self {
            instant_spawn: default_npcs_instant_spawn(),
            freeze_on_empty_map: default_npcs_freeze_on_empty_map(),
            chase_distance: default_npcs_chase_distance(),
            bored_timer: default_npcs_bored_timer(),
            act_rate: default_npcs_act_rate(),
            speed_0: default_npcs_speed_0(),
            speed_1: default_npcs_speed_1(),
            speed_2: default_npcs_speed_2(),
            speed_3: default_npcs_speed_3(),
            speed_4: default_npcs_speed_4(),
            speed_5: default_npcs_speed_5(),
            speed_6: default_npcs_speed_6(),
            talk_rate: default_npcs_talk_rate(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Sln {
    #[serde(default = "default_sln_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sln_url")]
    pub url: String,
    #[serde(default)]
    pub site: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default = "default_sln_server_name")]
    pub server_name: String,
    #[serde(default = "default_sln_rate")]
    pub rate: i32,
    #[serde(default)]
    pub zone: String,
}

fn default_sln_enabled() -> bool { false }
fn default_sln_url() -> String { "http://www.apollo-games.com/SLN/sln.php/".to_string() }
fn default_sln_server_name() -> String { "Untitled Server".to_string() }
fn default_sln_rate() -> i32 { 5 }

impl Default for Sln {
    fn default() -> Self {
        Self {
            enabled: default_sln_enabled(),
            url: default_sln_url(),
            site: Default::default(),
            hostname: Default::default(),
            server_name: default_sln_server_name(),
            rate: default_sln_rate(),
            zone: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Bank {
    #[serde(default = "default_bank_max_item_amount")]
    pub max_item_amount: i32,
    #[serde(default = "default_bank_base_size")]
    pub base_size: i32,
    #[serde(default = "default_bank_size_step")]
    pub size_step: i32,
    #[serde(default = "default_bank_max_upgrades")]
    pub max_upgrades: i32,
    #[serde(default = "default_bank_upgrade_base_cost")]
    pub upgrade_base_cost: i32,
    #[serde(default = "default_bank_upgrade_cost_step")]
    pub upgrade_cost_step: i32,
}

fn default_bank_max_item_amount() -> i32 { 200 }
fn default_bank_base_size() -> i32 { 25 }
fn default_bank_size_step() -> i32 { 5 }
fn default_bank_max_upgrades() -> i32 { 7 }
fn default_bank_upgrade_base_cost() -> i32 { 1000 }
fn default_bank_upgrade_cost_step() -> i32 { 1000 }

impl Default for Bank {
    fn default() -> Self {
        Self {
            max_item_amount: default_bank_max_item_amount(),
            base_size: default_bank_base_size(),
            size_step: default_bank_size_step(),
            max_upgrades: default_bank_max_upgrades(),
            upgrade_base_cost: default_bank_upgrade_base_cost(),
            upgrade_cost_step: default_bank_upgrade_cost_step(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Limits {
    #[serde(default = "default_limits_max_bank_gold")]
    pub max_bank_gold: i32,
    #[serde(default = "default_limits_max_item")]
    pub max_item: i32,
    #[serde(default = "default_limits_max_trade")]
    pub max_trade: i32,
    #[serde(default = "default_limits_max_chest")]
    pub max_chest: i32,
    #[serde(default = "default_limits_max_party_size")]
    pub max_party_size: i32,
}

fn default_limits_max_bank_gold() -> i32 { 2_000_000_000 }
fn default_limits_max_item() -> i32 { 2_000_000_000 }
fn default_limits_max_trade() -> i32 { 2_000_000_000 }
fn default_limits_max_chest() -> i32 { 10_000_000 }
fn default_limits_max_party_size() -> i32 { 9 }

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bank_gold: default_limits_max_bank_gold(),
            max_item: default_limits_max_item(),
            max_trade: default_limits_max_trade(),
            max_chest: default_limits_max_chest(),
            max_party_size: default_limits_max_party_size(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Combat {
    #[serde(default = "default_combat_weapon_ranges")]
    pub weapon_ranges: Vec<WeaponRange>,
    #[serde(default = "default_combat_enforce_weight")]
    pub enforce_weight: bool,
}

fn default_combat_enforce_weight() -> bool { true }
fn default_combat_weapon_ranges() -> Vec<WeaponRange> {
    vec![
        WeaponRange { weapon: 297, range: 5, arrows: true },
        WeaponRange { weapon: 316, range: 5, arrows: true },
        WeaponRange { weapon: 457, range: 5, arrows: true },
        WeaponRange { weapon: 365, range: 10, arrows: false },
    ]
}

impl Default for Combat {
    fn default() -> Self {
        Self {
            weapon_ranges: default_combat_weapon_ranges(),
            enforce_weight: default_combat_enforce_weight(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WeaponRange {
    pub weapon: i32,
    pub range: i32,
    #[serde(default)]
    pub arrows: bool,
}

#[derive(Debug, Deserialize)]
pub struct Board {
    #[serde(default = "default_board_max_posts")]
    pub max_posts: i32,
    #[serde(default = "default_board_max_user_posts")]
    pub max_user_posts: i32,
    #[serde(default = "default_board_max_recent_posts")]
    pub max_recent_posts: i32,
    #[serde(default = "default_board_recent_post_time")]
    pub recent_post_time: i32,
    #[serde(default = "default_board_max_subject_length")]
    pub max_subject_length: i32,
    #[serde(default = "default_board_max_post_length")]
    pub max_post_length: i32,
    #[serde(default = "default_board_date_posts")]
    pub date_posts: bool,
    #[serde(default = "default_board_admin_board")]
    pub admin_board: i32,
    #[serde(default = "default_board_admin_max_posts")]
    pub admin_max_posts: i32,
}

fn default_board_max_posts() -> i32 { 20 }
fn default_board_max_user_posts() -> i32 { 6 }
fn default_board_max_recent_posts() -> i32 { 2 }
fn default_board_recent_post_time() -> i32 { 30 }
fn default_board_max_subject_length() -> i32 { 32 }
fn default_board_max_post_length() -> i32 { 2048 }
fn default_board_date_posts() -> bool { true }
fn default_board_admin_board() -> i32 { 5 }
fn default_board_admin_max_posts() -> i32 { 100 }

impl Default for Board {
    fn default() -> Self {
        Self {
            max_posts: default_board_max_posts(),
            max_user_posts: default_board_max_user_posts(),
            max_recent_posts: default_board_max_recent_posts(),
            recent_post_time: default_board_recent_post_time(),
            max_subject_length: default_board_max_subject_length(),
            max_post_length: default_board_max_post_length(),
            date_posts: default_board_date_posts(),
            admin_board: default_board_admin_board(),
            admin_max_posts: default_board_admin_max_posts(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Chest {
    #[serde(default = "default_chest_slots")]
    pub slots: i32,
}

fn default_chest_slots() -> i32 { 5 }

impl Default for Chest {
    fn default() -> Self {
        Self {
            slots: default_chest_slots(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Quake {
    pub min_ticks: i32,
    pub max_ticks: i32,
    pub min_strength: i32,
    pub max_strength: i32,
}

impl Default for Quake {
    fn default() -> Self {
        Self {
            min_ticks: 0,
            max_ticks: 0,
            min_strength: 0,
            max_strength: 0,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Map {
    #[serde(default = "default_map_quakes")]
    pub quakes: Vec<Quake>,
    #[serde(default = "default_map_door_close_rate")]
    pub door_close_rate: i32,
    #[serde(default = "default_map_max_items")]
    pub max_items: i32,
}

fn default_map_quakes() -> Vec<Quake> {
    vec![
        Quake { min_ticks: 3, max_ticks: 29, min_strength: 0, max_strength: 2 },
        Quake { min_ticks: 3, max_ticks: 22, min_strength: 0, max_strength: 2 },
        Quake { min_ticks: 3, max_ticks: 15, min_strength: 4, max_strength: 6 },
        Quake { min_ticks: 2, max_ticks: 8, min_strength: 6, max_strength: 8 },
    ]
}
fn default_map_door_close_rate() -> i32 { 3 }
fn default_map_max_items() -> i32 { 250 }

impl Default for Map {
    fn default() -> Self {
        Self {
            quakes: default_map_quakes(),
            door_close_rate: default_map_door_close_rate(),
            max_items: default_map_max_items(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Jukebox {
    #[serde(default = "default_jukebox_cost")]
    pub cost: i32,
    #[serde(default = "default_jukebox_max_track_id")]
    pub max_track_id: i32,
    #[serde(default = "default_jukebox_track_timer")]
    pub track_timer: i32,
}

fn default_jukebox_cost() -> i32 { 25 }
fn default_jukebox_max_track_id() -> i32 { 20 }
fn default_jukebox_track_timer() -> i32 { 90 }

impl Default for Jukebox {
    fn default() -> Self {
        Self {
            cost: default_jukebox_cost(),
            max_track_id: default_jukebox_max_track_id(),
            track_timer: default_jukebox_track_timer(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Barber {
    #[serde(default = "default_barber_base_cost")]
    pub base_cost: i32,
    #[serde(default = "default_barber_cost_per_level")]
    pub cost_per_level: i32,
}

fn default_barber_base_cost() -> i32 { 0 }
fn default_barber_cost_per_level() -> i32 { 200 }

impl Default for Barber {
    fn default() -> Self {
        Self {
            base_cost: default_barber_base_cost(),
            cost_per_level: default_barber_cost_per_level(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Account {
    #[serde(default = "default_account_delay_time")]
    pub delay_time: i32,
    #[serde(default = "default_account_email_validation")]
    pub email_validation: bool,
    #[serde(default = "default_account_recovery")]
    pub recovery: bool,
    #[serde(default = "default_account_recovery_show_email")]
    pub recovery_show_email: bool,
    #[serde(default = "default_account_recovery_mask_email")]
    pub recovery_mask_email: bool,
    #[serde(default = "default_account_max_characters")]
    pub max_characters: usize,
}

fn default_account_delay_time() -> i32 { 10 }
fn default_account_email_validation() -> bool { true }
fn default_account_recovery() -> bool { true }
fn default_account_recovery_show_email() -> bool { true }
fn default_account_recovery_mask_email() -> bool { true }
fn default_account_max_characters() -> usize { 3 }

impl Default for Account {
    fn default() -> Self {
        Self {
            delay_time: default_account_delay_time(),
            email_validation: default_account_email_validation(),
            recovery: default_account_recovery(),
            recovery_show_email: default_account_recovery_show_email(),
            recovery_mask_email: default_account_recovery_mask_email(),
            max_characters: default_account_max_characters(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Guild {
    #[serde(default = "default_guild_min_players")]
    pub min_players: usize,
    #[serde(default = "default_guild_create_cost")]
    pub create_cost: i32,
    #[serde(default = "default_guild_recruit_cost")]
    pub recruit_cost: i32,
    #[serde(default = "default_guild_min_tag_length")]
    pub min_tag_length: usize,
    #[serde(default = "default_guild_max_tag_length")]
    pub max_tag_length: usize,
    #[serde(default = "default_guild_max_name_length")]
    pub max_name_length: usize,
    #[serde(default = "default_guild_max_description_length")]
    pub max_description_length: usize,
    #[serde(default = "default_guild_max_rank_length")]
    pub max_rank_length: usize,
    #[serde(default = "default_guild_default_leader_rank_name")]
    pub default_leader_rank_name: String,
    #[serde(default = "default_guild_default_recruiter_rank_name")]
    pub default_recruiter_rank_name: String,
    #[serde(default = "default_guild_default_new_member_rank_name")]
    pub default_new_member_rank_name: String,
    #[serde(default = "default_guild_min_deposit")]
    pub min_deposit: i32,
    #[serde(default = "default_guild_bank_max_gold")]
    pub bank_max_gold: i32,
}

fn default_guild_min_players() -> usize { 10 }
fn default_guild_create_cost() -> i32 { 50_000 }
fn default_guild_recruit_cost() -> i32 { 1000 }
fn default_guild_min_tag_length() -> usize { 2 }
fn default_guild_max_tag_length() -> usize { 3 }
fn default_guild_max_name_length() -> usize { 24 }
fn default_guild_max_description_length() -> usize { 240 }
fn default_guild_max_rank_length() -> usize { 16 }
fn default_guild_default_leader_rank_name() -> String { "Leader".to_string() }
fn default_guild_default_recruiter_rank_name() -> String { "Recruiter".to_string() }
fn default_guild_default_new_member_rank_name() -> String { "New Member".to_string() }
fn default_guild_min_deposit() -> i32 { 1000 }
fn default_guild_bank_max_gold() -> i32 { 2_000_000_000 }

impl Default for Guild {
    fn default() -> Self {
        Self {
            min_players: default_guild_min_players(),
            create_cost: default_guild_create_cost(),
            recruit_cost: default_guild_recruit_cost(),
            min_tag_length: default_guild_min_tag_length(),
            max_tag_length: default_guild_max_tag_length(),
            max_name_length: default_guild_max_name_length(),
            max_description_length: default_guild_max_description_length(),
            max_rank_length: default_guild_max_rank_length(),
            default_leader_rank_name: default_guild_default_leader_rank_name(),
            default_recruiter_rank_name: default_guild_default_recruiter_rank_name(),
            default_new_member_rank_name: default_guild_default_new_member_rank_name(),
            min_deposit: default_guild_min_deposit(),
            bank_max_gold: default_guild_bank_max_gold(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Marriage {
    #[serde(default = "default_marriage_approval_cost")]
    pub approval_cost: i32,
    #[serde(default = "default_marriage_divorce_cost")]
    pub divorce_cost: i32,
    #[serde(default = "default_marriage_female_armor_id")]
    pub female_armor_id: i32,
    #[serde(default = "default_marriage_male_armor_id")]
    pub male_armor_id: i32,
    #[serde(default = "default_marriage_min_level")]
    pub min_level: i32,
    #[serde(default = "default_marriage_mfx_id")]
    pub mfx_id: i32,
    #[serde(default = "default_marriage_ring_item_id")]
    pub ring_item_id: i32,
    #[serde(default = "default_marriage_ceremony_start_delay_seconds")]
    pub ceremony_start_delay_seconds: i32,
    #[serde(default = "default_marriage_celebration_effect_id")]
    pub celebration_effect_id: i32,
}

fn default_marriage_approval_cost() -> i32 { 500 }
fn default_marriage_divorce_cost() -> i32 { 10000 }
fn default_marriage_female_armor_id() -> i32 { 163 }
fn default_marriage_male_armor_id() -> i32 { 133 }
fn default_marriage_min_level() -> i32 { 5 }
fn default_marriage_mfx_id() -> i32 { 40 }
fn default_marriage_ring_item_id() -> i32 { 374 }
fn default_marriage_ceremony_start_delay_seconds() -> i32 { 20 }
fn default_marriage_celebration_effect_id() -> i32 { 1 }

impl Default for Marriage {
    fn default() -> Self {
        Self {
            approval_cost: default_marriage_approval_cost(),
            divorce_cost: default_marriage_divorce_cost(),
            female_armor_id: default_marriage_female_armor_id(),
            male_armor_id: default_marriage_male_armor_id(),
            min_level: default_marriage_min_level(),
            mfx_id: default_marriage_mfx_id(),
            ring_item_id: default_marriage_ring_item_id(),
            ceremony_start_delay_seconds: default_marriage_ceremony_start_delay_seconds(),
            celebration_effect_id: default_marriage_celebration_effect_id(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Evacuate {
    #[serde(default = "default_evacuate_sfx_id")]
    pub sfx_id: i32,
    #[serde(default = "default_evacuate_timer_seconds")]
    pub timer_seconds: i32,
    #[serde(default = "default_evacuate_timer_step")]
    pub timer_step: i32,
}

fn default_evacuate_sfx_id() -> i32 { 51 }
fn default_evacuate_timer_seconds() -> i32 { 60 }
fn default_evacuate_timer_step() -> i32 { 15 }

impl Default for Evacuate {
    fn default() -> Self {
        Self {
            sfx_id: default_evacuate_sfx_id(),
            timer_seconds: default_evacuate_timer_seconds(),
            timer_step: default_evacuate_timer_step(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Items {
    #[serde(default)]
    pub infinite_use_items: Vec<i32>,
    #[serde(default)]
    pub protected_items: Vec<i32>,
}

impl Default for Items {
    fn default() -> Self {
        Self {
            infinite_use_items: vec![],
            protected_items: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Smtp {
    #[serde(default)]
    pub from_name: String,
    #[serde(default)]
    pub from_address: String,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

fn default_smtp_port() -> u16 { 587 }

impl Default for Smtp {
    fn default() -> Self {
        Self {
            from_name: Default::default(),
            from_address: Default::default(),
            host: Default::default(),
            port: default_smtp_port(),
            username: Default::default(),
            password: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AutoPickup {
    #[serde(default = "default_auto_pickup_enabled")]
    pub enabled: bool,
    #[serde(default = "default_auto_pickup_rate")]
    pub rate: i32,
}

fn default_auto_pickup_enabled() -> bool { false }
fn default_auto_pickup_rate() -> i32 { 8 }

impl Default for AutoPickup {
    fn default() -> Self {
        Self {
            enabled: default_auto_pickup_enabled(),
            rate: default_auto_pickup_rate(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub server: Server,
    #[serde(default)]
    pub database: Database,
    #[serde(default)]
    pub account: Account,
    #[serde(default)]
    pub new_character: NewCharacter,
    #[serde(default)]
    pub jail: Jail,
    #[serde(default)]
    pub rescue: Rescue,
    #[serde(default)]
    pub world: World,
    #[serde(default)]
    pub character: Character,
    #[serde(default)]
    pub npcs: NPCs,
    #[serde(default)]
    pub sln: Sln,
    #[serde(default)]
    pub bank: Bank,
    #[serde(default)]
    pub limits: Limits,
    #[serde(default)]
    pub board: Board,
    #[serde(default)]
    pub chest: Chest,
    #[serde(default)]
    pub map: Map,
    #[serde(default)]
    pub combat: Combat,
    #[serde(default)]
    pub jukebox: Jukebox,
    #[serde(default)]
    pub barber: Barber,
    #[serde(default)]
    pub guild: Guild,
    #[serde(default)]
    pub marriage: Marriage,
    #[serde(default)]
    pub evacuate: Evacuate,
    #[serde(default)]
    pub items: Items,
    #[serde(default)]
    pub bard: Bard,
    #[serde(default)]
    pub smtp: Smtp,
    #[serde(default)]
    pub auto_pickup: AutoPickup,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .set_default("world.usage_rate", "60")
            .unwrap()
            .add_source(File::with_name("config/Config.toml"))
            .add_source(File::with_name("config/Config.local.toml").required(false))
            .build()?;

        s.try_deserialize()
    }

    pub fn reload() -> Result<Self, ConfigError> {
        Self::new()
    }
}
