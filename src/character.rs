use eo::{
    character::{AdminLevel, Gender, Race, SitState},
    data::{EOChar, EOInt, EOShort},
    net::{
        packets::client::character::Create, CharacterBaseStats, CharacterMapInfo,
        CharacterSecondaryStats, CharacterStats2, Item, PaperdollFull, Spell,
    },
    world::{Coords, Direction},
};

use chrono::prelude::*;
use mysql_async::{prelude::*, Conn, Params, Row, TxOpts};
use num_traits::FromPrimitive;

use crate::{player::PlayerHandle, utils, world::WorldHandle, SETTINGS};

#[derive(Debug, Clone, Default)]
pub struct Character {
    pub player_id: Option<EOShort>,
    pub player: Option<PlayerHandle>,
    pub world: Option<WorldHandle>,
    pub id: EOInt,
    pub account_id: EOInt,
    pub name: String,
    pub title: Option<String>,
    pub home: String,
    pub fiance: Option<String>,
    pub partner: Option<String>,
    pub admin_level: AdminLevel,
    pub class: EOChar,
    pub gender: Gender,
    pub race: Race,
    pub hair_style: EOShort,
    pub hair_color: EOShort,
    pub bank_max: EOInt,
    pub gold_bank: EOInt,
    pub guild_name: Option<String>,
    pub guild_tag: Option<String>,
    pub guild_rank_id: Option<EOChar>,
    pub guild_rank_string: Option<String>,
    pub paperdoll: PaperdollFull,
    pub level: EOChar,
    pub experience: EOInt,
    pub hp: EOShort,
    pub max_hp: EOShort,
    pub tp: EOShort,
    pub max_tp: EOShort,
    pub max_sp: EOShort,
    pub weight: EOInt,
    pub max_weight: EOInt,
    pub base_strength: EOShort,
    pub base_intelligence: EOShort,
    pub base_wisdom: EOShort,
    pub base_agility: EOShort,
    pub base_constitution: EOShort,
    pub base_charisma: EOShort,
    pub adj_strength: EOShort,
    pub adj_intelligence: EOShort,
    pub adj_wisdom: EOShort,
    pub adj_agility: EOShort,
    pub adj_constitution: EOShort,
    pub adj_charisma: EOShort,
    pub stat_points: EOShort,
    pub skill_points: EOShort,
    pub karma: EOShort,
    pub usage: EOInt,
    pub min_damage: EOShort,
    pub max_damage: EOShort,
    pub accuracy: EOShort,
    pub evasion: EOShort,
    pub armor: EOShort,
    pub map_id: EOShort,
    pub coords: Coords,
    pub direction: Direction,
    pub sit_state: SitState,
    pub hidden: bool,
    pub items: Vec<Item>,
    pub bank: Vec<Item>,
    pub spells: Vec<Spell>,
    pub logged_in_at: Option<DateTime<Utc>>,
}

impl Character {
    pub fn from_creation(account_id: EOInt, create: &Create) -> Self {
        let mut character = Character::default();
        character.account_id = account_id;
        character.gender = create.gender;
        character.hair_style = create.hair_style;
        character.hair_color = create.hair_color;
        character.race = create.race;
        character.name = create.name.clone();
        character
    }

    pub fn is_in_range(&self, coords: Coords) -> bool {
        utils::in_range(
            self.coords.x.into(),
            self.coords.y.into(),
            coords.x.into(),
            coords.y.into(),
        )
    }

    pub fn to_map_info(&self) -> CharacterMapInfo {
        CharacterMapInfo {
            name: self.name.clone(),
            id: self.player_id.expect("Character has no player id"),
            map_id: self.map_id,
            coords: self.coords,
            direction: self.direction,
            class_id: self.class,
            guild_tag: match self.guild_tag {
                Some(ref tag) => tag.to_string(),
                None => String::new(),
            },
            level: self.level,
            gender: self.gender,
            hair_style: self.hair_style as EOChar,
            hair_color: self.hair_color as EOChar,
            race: self.race,
            max_hp: self.max_hp,
            hp: self.hp,
            max_tp: self.max_tp,
            tp: self.tp,
            paperdoll: self.paperdoll.to_paperdoll_b000a0hsw(),
            sit_state: self.sit_state,
            invisible: self.hidden,
        }
    }

    pub async fn calculate_stats(&mut self) {
        let world = self.world.as_ref().expect("Character has no world");
        match world.get_class(self.class).await {
            Ok(class) => {
                self.adj_strength = self.base_strength + class.strength;
                self.adj_intelligence = self.base_intelligence + class.intelligence;
                self.adj_wisdom = self.base_wisdom + class.wisdom;
                self.adj_agility = self.base_agility + class.agility;
                self.adj_constitution = self.base_constitution + class.constitution;
                self.adj_charisma = self.base_charisma + class.charisma;
            }
            _ => {}
        }

        self.max_weight = 70;
        self.weight = 0;
        self.max_hp = 0;
        self.max_tp = 0;
        self.min_damage = 0;
        self.max_damage = 0;
        self.accuracy = 0;
        self.evasion = 0;
        self.armor = 0;
        self.max_sp = 0;

        for item in &self.items {
            if item.id == 0 {
                continue;
            }

            match world.get_item(item.id).await {
                Ok(record) => {
                    self.weight += record.weight as EOInt * item.amount;
                    if self.weight >= 250 {
                        break;
                    }
                }
                _ => {}
            }
        }

        for item in self.paperdoll {
            if item == 0 {
                continue;
            }

            match world.get_item(item).await {
                Ok(item) => {
                    self.weight += item.weight as EOInt;
                    self.max_hp += item.hp;
                    self.max_tp += item.tp;
                    self.min_damage += item.min_damage;
                    self.max_damage += item.max_damage;
                    self.accuracy += item.accuracy;
                    self.evasion += item.evade;
                    self.armor += item.armor;
                    self.adj_strength += item.strength as EOShort;
                    self.adj_intelligence += item.intelligence as EOShort;
                    self.adj_wisdom += item.wisdom as EOShort;
                    self.adj_agility += item.agility as EOShort;
                    self.adj_constitution += item.constitution as EOShort;
                    self.adj_charisma += item.charisma as EOShort;
                }
                _ => {}
            }
        }

        if self.weight > 250 {
            self.weight = 250;
        }
    }

    pub async fn load(
        conn: &mut Conn,
        id: EOInt,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut character = Character::default();
        let mut row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("sql/get_character.sql"),
                params! {
                    "character_id" => id,
                },
            )
            .await?
        {
            Some(row) => row,
            _ => {
                panic!(
                    "Attempting to load character that doesn't exist! ID: {}",
                    id
                );
            }
        };

        character.id = id;
        character.account_id = row.take("account_id").unwrap();
        character.name = row.take("name").unwrap();
        character.title = row.take("title").unwrap();
        character.home = row.take("home").unwrap();
        character.fiance = row.take("fiance").unwrap();
        character.partner = row.take("partner").unwrap();
        character.admin_level = AdminLevel::from_i32(row.take("admin_level").unwrap()).unwrap();
        character.class = row.take("class").unwrap();
        character.gender = Gender::from_i32(row.take("gender").unwrap()).unwrap();
        character.race = Race::from_i32(row.take("race").unwrap()).unwrap();
        character.hair_style = row.take("hair_style").unwrap();
        character.hair_color = row.take("hair_color").unwrap();
        character.bank_max = row.take("bank_max").unwrap();
        character.gold_bank = row.take("gold_bank").unwrap();
        character.guild_rank_id = row.take("guild_rank_id").unwrap();
        character.guild_rank_string = row.take("guild_rank_string").unwrap();
        character.paperdoll.boots = row.take("boots").unwrap();
        character.paperdoll.accessory = row.take("accessory").unwrap();
        character.paperdoll.gloves = row.take("gloves").unwrap();
        character.paperdoll.belt = row.take("belt").unwrap();
        character.paperdoll.armor = row.take("armor").unwrap();
        character.paperdoll.hat = row.take("hat").unwrap();
        character.paperdoll.shield = row.take("shield").unwrap();
        character.paperdoll.weapon = row.take("weapon").unwrap();
        character.paperdoll.rings[0] = row.take("ring").unwrap();
        character.paperdoll.rings[1] = row.take("ring2").unwrap();
        character.paperdoll.armlets[0] = row.take("armlet").unwrap();
        character.paperdoll.armlets[1] = row.take("armlet2").unwrap();
        character.paperdoll.bracers[0] = row.take("bracer").unwrap();
        character.paperdoll.bracers[1] = row.take("bracer2").unwrap();
        character.level = row.take("level").unwrap();
        character.experience = row.take("experience").unwrap();
        character.hp = row.take("hp").unwrap();
        character.tp = row.take("tp").unwrap();
        character.base_strength = row.take("strength").unwrap();
        character.base_intelligence = row.take("intelligence").unwrap();
        character.base_wisdom = row.take("wisdom").unwrap();
        character.base_agility = row.take("agility").unwrap();
        character.base_constitution = row.take("constitution").unwrap();
        character.base_charisma = row.take("charisma").unwrap();
        character.stat_points = row.take("stat_points").unwrap();
        character.skill_points = row.take("skill_points").unwrap();
        character.karma = row.take("karma").unwrap();
        character.usage = row.take("usage").unwrap();
        character.map_id = row.take("map").unwrap();
        character.coords.x = row.take("x").unwrap();
        character.coords.y = row.take("y").unwrap();
        character.direction = Direction::from_i32(row.take("direction").unwrap()).unwrap();
        character.sit_state = SitState::from_i32(row.take("sitting").unwrap()).unwrap();
        character.hidden = row.take::<u32, &str>("hidden").unwrap() == 1;
        character.guild_name = row.take("guild_name").unwrap();
        character.guild_tag = row.take("tag").unwrap();

        character.items = conn
            .exec_map(
                include_str!("sql/get_character_inventory.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        character.bank = conn
            .exec_map(
                include_str!("sql/get_character_bank.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

        character.spells = conn
            .exec_map(
                include_str!("sql/get_character_spells.sql"),
                params! {
                    "character_id" => id,
                },
                |mut row: Row| Spell {
                    id: row.take(0).unwrap(),
                    level: row.take(1).unwrap(),
                },
            )
            .await?;

        Ok(character)
    }

    pub async fn save(
        &mut self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.id > 0 {
            self.update(conn).await
        } else {
            self.create(conn).await
        }
    }

    async fn create(
        &mut self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            include_str!("./sql/create_character.sql"),
            params! {
                "account_id" => &(self.account_id as u32),
                "name" => &self.name,
                "home" => &SETTINGS.new_character.home,
                "gender" => &(self.gender as u32),
                "race" => &(self.race as u32),
                "hair_style" => &(self.hair_style as u32),
                "hair_color" => &(self.hair_color as u32),
                "bank_max" => &0_u32, // TODO: figure out bank max
            },
        )
        .await?;

        self.id = tx.last_insert_id().unwrap() as EOInt;

        tx.exec_drop(
            r"INSERT INTO `Paperdoll` (
                `character_id`
            ) VALUES (:character_id);",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"INSERT INTO `Position` (
                `character_id`,
                `map`,
                `x`,
                `y`,
                `direction`
            ) VALUES (
                :character_id,
                :map,
                :x,
                :y,
                :direction
            );",
            params! {
                "character_id" => &self.id,
                "map" => &SETTINGS.new_character.spawn_map,
                "x" => &SETTINGS.new_character.spawn_x,
                "y" => &SETTINGS.new_character.spawn_y,
                "direction" => &SETTINGS.new_character.spawn_direction,
            },
        )
        .await?;

        tx.exec_drop(
            r" INSERT INTO `Stats` (`character_id`)
            VALUES (:character_id);",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update(
        &self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            include_str!("./sql/update_character.sql"),
            params! {
                "character_id" => self.id,
                "title" => &self.title,
                "home" => &self.home,
                "fiance" => &self.fiance,
                "partner" => &self.partner,
                "admin_level" => self.admin_level as u32,
                "class" => self.class as u32,
                "gender" => self.gender as u32,
                "race" => self.race as u32,
                "hair_style" => self.hair_style as u32,
                "hair_color" => self.hair_color as u32,
                "bank_max" => self.bank_max as u32,
                "gold_bank" => self.gold_bank as u32,
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("./sql/update_paperdoll.sql"),
            params! {
                "character_id" => self.id,
                "boots" => self.paperdoll.boots as u32,
                "accessory" => self.paperdoll.accessory as u32,
                "gloves" => self.paperdoll.gloves as u32,
                "belt" => self.paperdoll.belt as u32,
                "armor" => self.paperdoll.armor as u32,
                "necklace" => self.paperdoll.necklace as u32,
                "hat" => self.paperdoll.hat as u32,
                "shield" => self.paperdoll.shield as u32,
                "weapon" => self.paperdoll.weapon as u32,
                "ring" => self.paperdoll.rings[0] as u32,
                "ring2" => self.paperdoll.rings[1] as u32,
                "armlet" => self.paperdoll.armlets[0] as u32,
                "armlet2" => self.paperdoll.armlets[1] as u32,
                "bracer" => self.paperdoll.bracers[0] as u32,
                "bracer2" => self.paperdoll.bracers[1] as u32,
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("./sql/update_position.sql"),
            params! {
                "character_id" => self.id,
                "map_id" => self.map_id as u32,
                "x" => self.coords.x as u32,
                "y" => self.coords.y as u32,
                "direction" => self.direction as u32,
                "sitting" => self.sit_state as u32,
                "hidden" => if self.hidden { 1 } else { 0 },
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("./sql/update_stats.sql"),
            params! {
                "character_id" => self.id,
                "level" => self.level as u32,
                "experience" => self.experience as u32,
                "hp" => self.hp as u32,
                "tp" => self.tp as u32,
                "strength" => self.base_strength as u32,
                "intelligence" => self.base_intelligence as u32,
                "wisdom" => self.base_wisdom as u32,
                "agility" => self.base_agility as u32,
                "constitution" => self.base_constitution as u32,
                "charisma" => self.base_charisma as u32,
                "stat_points" => self.stat_points as u32,
                "skill_points" => self.skill_points as u32,
                "karma" => self.karma as u32,
                "usage" => self.usage,
            },
        )
        .await?;

        // TODO: save inventory/bank/spells

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete(
        &self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            r"DELETE FROM `Stats` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Spell` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Position` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Paperdoll` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Inventory` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Bank` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Character` WHERE `id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub fn get_character_stats_2(&self) -> CharacterStats2 {
        CharacterStats2 {
            hp: self.hp,
            max_hp: self.max_hp,
            tp: self.tp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            stat_points: self.stat_points,
            skill_points: self.skill_points,
            karma: self.karma,
            secondary: CharacterSecondaryStats {
                min_damage: self.min_damage,
                max_damage: self.max_damage,
                accuracy: self.accuracy,
                evasion: self.evasion,
                armor: self.armor,
            },
            base: CharacterBaseStats {
                strength: self.adj_strength,
                intelligence: self.adj_intelligence,
                wisdom: self.adj_wisdom,
                agility: self.adj_agility,
                constitution: self.adj_constitution,
                charisma: self.adj_charisma,
            },
        }
    }
}
