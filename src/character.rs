use std::cmp;

use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable, StreamBuilder, MAX2},
    protocol::{
        client::character::Create, server::paperdoll, AdminLevel, BigCoords, CharacterBaseStats,
        CharacterBaseStats2, CharacterMapInfo, CharacterSecondaryStats, CharacterStats2, Coords,
        Direction, Gender, Item, ItemCharacterStats, PacketAction, PacketFamily,
        PaperdollB000a0hsw, PaperdollBahws, PaperdollFull, PaperdollIcon, SitState, Skin, Spell,
    },
    pubs::EifItemType,
};

use chrono::prelude::*;
use evalexpr::{context_map, eval_float_with_context};
use mysql_async::{prelude::*, Conn, Params, Row, TxOpts};

use crate::{player::PlayerHandle, utils, CLASS_DB, FORMULAS, ITEM_DB, SETTINGS};

pub enum PaperdollSlot {
    Boots,
    Accessory,
    Gloves,
    Belt,
    Armor,
    Necklace,
    Hat,
    Shield,
    Weapon,
    Ring1,
    Ring2,
    Armlet1,
    Armlet2,
    Bracer1,
    Bracer2,
}

impl PaperdollSlot {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(PaperdollSlot::Boots),
            1 => Some(PaperdollSlot::Accessory),
            2 => Some(PaperdollSlot::Gloves),
            3 => Some(PaperdollSlot::Belt),
            4 => Some(PaperdollSlot::Armor),
            5 => Some(PaperdollSlot::Necklace),
            6 => Some(PaperdollSlot::Hat),
            7 => Some(PaperdollSlot::Shield),
            8 => Some(PaperdollSlot::Weapon),
            9 => Some(PaperdollSlot::Ring1),
            10 => Some(PaperdollSlot::Ring2),
            11 => Some(PaperdollSlot::Armlet1),
            12 => Some(PaperdollSlot::Armlet2),
            13 => Some(PaperdollSlot::Bracer1),
            14 => Some(PaperdollSlot::Bracer2),
            _ => None,
        }
    }
    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            PaperdollSlot::Boots
                | PaperdollSlot::Armor
                | PaperdollSlot::Hat
                | PaperdollSlot::Shield
                | PaperdollSlot::Weapon
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct Character {
    pub player_id: Option<EOShort>,
    pub player: Option<PlayerHandle>,
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
    pub skin: Skin,
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
    pub weight: EOChar,
    pub max_weight: EOChar,
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
        Character {
            account_id,
            gender: create.gender,
            hair_style: create.hairstyle,
            hair_color: create.haircolor,
            skin: create.skin,
            name: create.name.clone(),
            ..Default::default()
        }
    }

    pub fn destroy_equipment(&mut self, slot: &PaperdollSlot) {
        match slot {
            PaperdollSlot::Boots => self.paperdoll.boots = 0,
            PaperdollSlot::Accessory => self.paperdoll.accessory = 0,
            PaperdollSlot::Gloves => self.paperdoll.gloves = 0,
            PaperdollSlot::Belt => self.paperdoll.belt = 0,
            PaperdollSlot::Armor => self.paperdoll.armor = 0,
            PaperdollSlot::Necklace => self.paperdoll.necklace = 0,
            PaperdollSlot::Hat => self.paperdoll.hat = 0,
            PaperdollSlot::Shield => self.paperdoll.shield = 0,
            PaperdollSlot::Weapon => self.paperdoll.weapon = 0,
            PaperdollSlot::Ring1 => self.paperdoll.ring[0] = 0,
            PaperdollSlot::Ring2 => self.paperdoll.ring[1] = 0,
            PaperdollSlot::Armlet1 => self.paperdoll.armlet[0] = 0,
            PaperdollSlot::Armlet2 => self.paperdoll.armlet[1] = 0,
            PaperdollSlot::Bracer1 => self.paperdoll.bracer[0] = 0,
            PaperdollSlot::Bracer2 => self.paperdoll.bracer[1] = 0,
        }
    }

    pub fn get_paperdoll_array(&self) -> [EOShort; 15] {
        [
            self.paperdoll.boots,
            self.paperdoll.accessory,
            self.paperdoll.gloves,
            self.paperdoll.belt,
            self.paperdoll.armor,
            self.paperdoll.necklace,
            self.paperdoll.hat,
            self.paperdoll.shield,
            self.paperdoll.weapon,
            self.paperdoll.ring[0],
            self.paperdoll.ring[1],
            self.paperdoll.armlet[0],
            self.paperdoll.armlet[1],
            self.paperdoll.bracer[0],
            self.paperdoll.bracer[1],
        ]
    }

    pub fn get_hp_percentage(&self) -> EOChar {
        let percent = (self.hp as f32 / self.max_hp as f32) * 100.0;
        percent.floor() as EOChar
    }

    pub fn heal(&mut self, amount: EOShort) -> EOShort {
        let amount = cmp::min(amount, self.max_hp - self.hp);
        self.hp += amount;
        amount
    }

    pub fn tp_heal(&mut self, amount: EOShort) -> EOShort {
        let amount = cmp::min(amount, self.max_tp - self.tp);
        self.tp += amount;
        amount
    }

    pub fn calculate_stats(&mut self) {
        let class = &CLASS_DB.classes[(self.class - 1) as usize];

        self.adj_strength = self.base_strength + class.str;
        self.adj_intelligence = self.base_intelligence + class.intl;
        self.adj_wisdom = self.base_wisdom + class.wis;
        self.adj_agility = self.base_agility + class.agi;
        self.adj_constitution = self.base_constitution + class.con;
        self.adj_charisma = self.base_charisma + class.cha;

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

            let record = &ITEM_DB.items[(item.id - 1) as usize];
            self.weight += (record.weight as EOInt * item.amount) as EOChar;
            if self.weight >= 250 {
                break;
            }
        }

        let paperdoll_items = vec![
            self.paperdoll.boots,
            self.paperdoll.accessory,
            self.paperdoll.gloves,
            self.paperdoll.belt,
            self.paperdoll.armor,
            self.paperdoll.necklace,
            self.paperdoll.hat,
            self.paperdoll.shield,
            self.paperdoll.weapon,
            self.paperdoll.ring[0],
            self.paperdoll.ring[1],
            self.paperdoll.armlet[0],
            self.paperdoll.armlet[1],
            self.paperdoll.bracer[0],
            self.paperdoll.bracer[1],
        ];

        for item_id in paperdoll_items {
            if item_id == 0 {
                continue;
            }

            let item = &ITEM_DB.items[(item_id - 1) as usize];
            self.weight += item.weight;
            self.max_hp += item.hp;
            self.max_tp += item.tp;
            self.min_damage += item.min_damage;
            self.max_damage += item.max_damage;
            self.accuracy += item.accuracy;
            self.evasion += item.evade;
            self.armor += item.armor;
            self.adj_strength += item.str as EOShort;
            self.adj_intelligence += item.intl as EOShort;
            self.adj_wisdom += item.wis as EOShort;
            self.adj_agility += item.agi as EOShort;
            self.adj_constitution += item.con as EOShort;
            self.adj_charisma += item.cha as EOShort;
        }

        if self.weight > 250 {
            self.weight = 250;
        }

        let context = match context_map! {
            "base_str" => self.base_strength as i64,
            "base_int" => self.base_intelligence as i64,
            "base_wis" => self.base_wisdom as i64,
            "base_agi" => self.base_agility as i64,
            "base_con" => self.base_constitution as i64,
            "base_cha" => self.base_charisma as i64,
            "str" => self.adj_strength as i64,
            "int" => self.adj_intelligence as i64,
            "wis" => self.adj_wisdom as i64,
            "agi" => self.adj_agility as i64,
            "con" => self.adj_constitution as i64,
            "cha" => self.adj_charisma as i64,
            "level" => self.level as i64,
        } {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to generate formula context: {}", e);
                return;
            }
        };

        self.max_hp += match eval_float_with_context(&FORMULAS.hp, &context) {
            Ok(max_hp) => max_hp.floor() as EOShort,
            Err(e) => {
                error!("Failed to calculate max_hp: {}", e);
                10
            }
        };

        self.max_tp += match eval_float_with_context(&FORMULAS.tp, &context) {
            Ok(max_tp) => cmp::min(max_tp.floor() as EOInt, MAX2 - 1) as EOShort,
            Err(e) => {
                error!("Failed to calculate max_tp: {}", e);
                10
            }
        };

        self.max_sp += match eval_float_with_context(&FORMULAS.sp, &context) {
            Ok(max_sp) => cmp::min(max_sp.floor() as EOInt, MAX2 - 1) as EOShort,
            Err(e) => {
                error!("Failed to calculate max_sp: {}", e);
                20
            }
        };

        self.max_weight = match eval_float_with_context(&FORMULAS.max_weight, &context) {
            Ok(max_weight) => cmp::min(max_weight.floor() as EOInt, 250) as EOChar,
            Err(e) => {
                error!("Failed to calculate max_weight: {}", e);
                70
            }
        };

        let class_formulas = &FORMULAS.classes[class.r#type as usize];
        let damage = match eval_float_with_context(&class_formulas.damage, &context) {
            Ok(damage) => damage.floor() as EOShort,
            Err(e) => {
                error!("Failed to calculate damage: {}", e);
                1
            }
        };

        self.min_damage += damage;
        self.max_damage += damage;

        self.accuracy += match eval_float_with_context(&class_formulas.accuracy, &context) {
            Ok(accuracy) => accuracy.floor() as EOShort,
            Err(e) => {
                error!("Failed to calculate accuracy: {}", e);
                0
            }
        };

        self.armor += match eval_float_with_context(&class_formulas.defense, &context) {
            Ok(armor) => armor.floor() as EOShort,
            Err(e) => {
                error!("Failed to calculate armor: {}", e);
                0
            }
        };

        self.evasion += match eval_float_with_context(&class_formulas.evade, &context) {
            Ok(evasion) => evasion.floor() as EOShort,
            Err(e) => {
                error!("Failed to calculate evasion: {}", e);
                0
            }
        };

        if self.min_damage == 0 {
            self.min_damage = 1;
        }

        if self.max_damage == 0 {
            self.max_damage = 1;
        }
    }

    pub fn get_icon(&self) -> PaperdollIcon {
        // TODO: group stuff

        match self.admin_level {
            AdminLevel::Player | AdminLevel::Guide | AdminLevel::Guardian => PaperdollIcon::Player,
            AdminLevel::Gm => PaperdollIcon::Gm,
            AdminLevel::Hgm | AdminLevel::God => PaperdollIcon::Hgm,
        }
    }

    pub fn is_in_range(&self, coords: &Coords) -> bool {
        utils::in_range(&self.coords, coords)
    }

    pub fn can_hold(&self, item_id: EOShort, max_amount: EOInt) -> EOInt {
        let item = ITEM_DB.items.get(item_id as usize - 1);

        if item.is_none() {
            return max_amount;
        }

        let item = item.unwrap();

        if item.weight == 0 {
            return max_amount;
        }

        let remaining_weight = self.max_weight - self.weight;
        let max_items = (remaining_weight as f64 / item.weight as f64).floor();
        cmp::min(max_items as EOInt, max_amount)
    }

    pub fn add_item(&mut self, item_id: EOShort, amount: EOInt) {
        let existing_item = self.items.iter_mut().find(|item| item.id == item_id);

        if let Some(existing_item) = existing_item {
            existing_item.amount += amount;
        } else {
            self.items.push(Item {
                id: item_id,
                amount,
            });
        }

        if let Some(item) = ITEM_DB.items.get(item_id as usize - 1) {
            self.weight += (item.weight as EOInt * amount) as EOChar;
        }
    }

    pub fn remove_item(&mut self, item_id: EOShort, amount: EOInt) {
        let existing_item = match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return,
        };

        if existing_item.amount <= amount {
            self.items.retain(|item| item.id != item_id);
        } else {
            existing_item.amount -= amount;
        }

        if let Some(item) = ITEM_DB.items.get(item_id as usize - 1) {
            self.weight -= (item.weight as EOInt * amount) as EOChar;
        }
    }

    pub fn equip(&mut self, item_id: EOShort, sub_loc: EOChar) -> bool {
        if sub_loc > 1 {
            return false;
        }

        let existing_item = match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return false,
        };

        let item_record = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => return false,
        };

        if item_record.r#type == EifItemType::Armor && item_record.spec2 != self.gender.to_char() {
            return false;
        }

        if (self.level as EOShort) < item_record.level_req
            || self.adj_strength < item_record.str_req
            || self.adj_intelligence < item_record.int_req
            || self.adj_wisdom < item_record.wis_req
            || self.adj_agility < item_record.agi_req
            || self.adj_constitution < item_record.con_req
            || self.adj_charisma < item_record.cha_req
        {
            return false;
        }

        if item_record.class_req != 0 && item_record.class_req != self.class as EOShort {
            let reply = paperdoll::Ping {
                class_id: self.class,
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            self.player.as_ref().unwrap().send(
                PacketAction::Ping,
                PacketFamily::Paperdoll,
                builder.get(),
            );
            return false;
        }

        match item_record.r#type {
            EifItemType::Weapon => {
                if self.paperdoll.weapon != 0 {
                    return false;
                }
                self.paperdoll.weapon = item_id
            }
            EifItemType::Shield => {
                if self.paperdoll.shield != 0 {
                    return false;
                }
                self.paperdoll.shield = item_id
            }
            EifItemType::Armor => {
                if self.paperdoll.armor != 0 {
                    return false;
                }
                self.paperdoll.armor = item_id
            }
            EifItemType::Hat => {
                if self.paperdoll.hat != 0 {
                    return false;
                }
                self.paperdoll.hat = item_id
            }
            EifItemType::Boots => {
                if self.paperdoll.boots != 0 {
                    return false;
                }
                self.paperdoll.boots = item_id
            }
            EifItemType::Gloves => {
                if self.paperdoll.gloves != 0 {
                    return false;
                }
                self.paperdoll.gloves = item_id
            }
            EifItemType::Accessory => {
                if self.paperdoll.accessory != 0 {
                    return false;
                }
                self.paperdoll.accessory = item_id
            }
            EifItemType::Belt => {
                if self.paperdoll.belt != 0 {
                    return false;
                }
                self.paperdoll.belt = item_id
            }
            EifItemType::Necklace => {
                if self.paperdoll.necklace != 0 {
                    return false;
                }
                self.paperdoll.necklace = item_id
            }
            EifItemType::Ring => {
                if self.paperdoll.ring[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.ring[sub_loc as usize] = item_id
            }
            EifItemType::Armlet => {
                if self.paperdoll.armlet[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.armlet[sub_loc as usize] = item_id
            }
            EifItemType::Bracer => {
                if self.paperdoll.bracer[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.bracer[sub_loc as usize] = item_id
            }
            _ => {
                warn!(
                    "{} tried to equip an invalid item type: {:?}",
                    self.name, item_record.r#type
                );
                return false;
            }
        }

        if existing_item.amount <= 1 {
            self.items.retain(|item| item.id != item_id);
        } else {
            existing_item.amount -= 1;
        }

        self.calculate_stats();
        true
    }

    pub fn get_paperdoll_bahws(&self) -> PaperdollBahws {
        PaperdollBahws {
            boots: match self.paperdoll.boots {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.boots as usize - 1) {
                    Some(item) => item.spec1 as EOShort,
                    None => 0,
                },
            },
            armor: match self.paperdoll.armor {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.armor as usize - 1) {
                    Some(item) => item.spec1 as EOShort,
                    None => 0,
                },
            },
            hat: match self.paperdoll.hat {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.hat as usize - 1) {
                    Some(item) => item.spec1 as EOShort,
                    None => 0,
                },
            },
            weapon: match self.paperdoll.weapon {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.weapon as usize - 1) {
                    Some(item) => item.spec1 as EOShort,
                    None => 0,
                },
            },
            shield: match self.paperdoll.shield {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.shield as usize - 1) {
                    Some(item) => item.spec1 as EOShort,
                    None => 0,
                },
            },
        }
    }

    pub fn get_paperdoll_b000a0hsw(&self) -> PaperdollB000a0hsw {
        let paperdoll = self.get_paperdoll_bahws();
        PaperdollB000a0hsw {
            boots: paperdoll.boots,
            armor: paperdoll.armor,
            hat: paperdoll.hat,
            weapon: paperdoll.weapon,
            shield: paperdoll.shield,
        }
    }

    pub fn unequip(&mut self, item_id: EOShort, sub_loc: EOChar) -> bool {
        if sub_loc > 1 {
            return false;
        }

        let item_record = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => return false,
        };

        match item_record.r#type {
            EifItemType::Weapon => {
                if self.paperdoll.weapon != item_id {
                    return false;
                }
                self.paperdoll.weapon = 0;
            }
            EifItemType::Shield => {
                if self.paperdoll.shield != item_id {
                    return false;
                }
                self.paperdoll.shield = 0;
            }
            EifItemType::Armor => {
                if self.paperdoll.armor != item_id {
                    return false;
                }
                self.paperdoll.armor = 0;
            }
            EifItemType::Hat => {
                if self.paperdoll.hat != item_id {
                    return false;
                }
                self.paperdoll.hat = 0;
            }
            EifItemType::Boots => {
                if self.paperdoll.boots != item_id {
                    return false;
                }
                self.paperdoll.boots = 0;
            }
            EifItemType::Gloves => {
                if self.paperdoll.gloves != item_id {
                    return false;
                }
                self.paperdoll.gloves = 0;
            }
            EifItemType::Accessory => {
                if self.paperdoll.accessory != item_id {
                    return false;
                }
                self.paperdoll.accessory = 0;
            }
            EifItemType::Belt => {
                if self.paperdoll.belt != item_id {
                    return false;
                }
                self.paperdoll.belt = 0;
            }
            EifItemType::Necklace => {
                if self.paperdoll.necklace != item_id {
                    return false;
                }
                self.paperdoll.necklace = 0;
            }
            EifItemType::Ring => {
                if self.paperdoll.ring[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.ring[sub_loc as usize] = 0;
            }
            EifItemType::Armlet => {
                if self.paperdoll.armlet[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.armlet[sub_loc as usize] = 0;
            }
            EifItemType::Bracer => {
                if self.paperdoll.bracer[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.bracer[sub_loc as usize] = 0;
            }
            _ => {
                warn!(
                    "{} tried to unequip an invalid item type: {:?}",
                    self.name, item_record.r#type
                );
                return false;
            }
        }

        match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => {
                item.amount += 1;
            }
            None => {
                self.items.push(Item {
                    id: item_id,
                    amount: 1,
                });
            }
        }

        self.calculate_stats();
        true
    }

    pub fn to_map_info(&self) -> CharacterMapInfo {
        CharacterMapInfo {
            name: self.name.clone(),
            id: self.player_id.expect("Character has no player id"),
            map_id: self.map_id,
            coords: BigCoords {
                x: self.coords.x.into(),
                y: self.coords.y.into(),
            },
            direction: self.direction,
            class_id: self.class,
            guild_tag: match self.guild_tag {
                Some(ref tag) => tag.to_string(),
                None => String::new(),
            },
            level: self.level,
            gender: self.gender,
            hairstyle: self.hair_style as EOChar,
            haircolor: self.hair_color as EOChar,
            skin_id: self.skin,
            max_hp: self.max_hp,
            hp: self.hp,
            max_tp: self.max_tp,
            tp: self.tp,
            paperdoll: self.get_paperdoll_b000a0hsw(),
            sit_state: self.sit_state,
            invisible: EOChar::from(self.hidden),
            animation: None,
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
        character.admin_level = AdminLevel::from_char(row.take("admin_level").unwrap()).unwrap();
        character.class = row.take("class").unwrap();
        character.gender = Gender::from_char(row.take("gender").unwrap()).unwrap();
        character.skin = Skin::from_char(row.take("race").unwrap()).unwrap();
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
        character.paperdoll.ring[0] = row.take("ring").unwrap();
        character.paperdoll.ring[1] = row.take("ring2").unwrap();
        character.paperdoll.armlet[0] = row.take("armlet").unwrap();
        character.paperdoll.armlet[1] = row.take("armlet2").unwrap();
        character.paperdoll.bracer[0] = row.take("bracer").unwrap();
        character.paperdoll.bracer[1] = row.take("bracer2").unwrap();
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
        character.direction = Direction::from_char(row.take("direction").unwrap()).unwrap();
        character.sit_state = SitState::from_char(row.take("sitting").unwrap()).unwrap();
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
                "account_id" => &self.account_id,
                "name" => &self.name,
                "home" => &SETTINGS.new_character.home,
                "gender" => &(self.gender as u32),
                "race" => &(self.skin as u32),
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
        let old_items = conn
            .exec_map(
                include_str!("sql/get_character_inventory.sql"),
                params! {
                    "character_id" => self.id,
                },
                |mut row: Row| Item {
                    id: row.take(0).unwrap(),
                    amount: row.take(1).unwrap(),
                },
            )
            .await?;

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
                "race" => self.skin as u32,
                "hair_style" => self.hair_style as u32,
                "hair_color" => self.hair_color as u32,
                "bank_max" => self.bank_max,
                "gold_bank" => self.gold_bank,
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
                "ring" => self.paperdoll.ring[0] as u32,
                "ring2" => self.paperdoll.ring[1] as u32,
                "armlet" => self.paperdoll.armlet[0] as u32,
                "armlet2" => self.paperdoll.armlet[1] as u32,
                "bracer" => self.paperdoll.bracer[0] as u32,
                "bracer2" => self.paperdoll.bracer[1] as u32,
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
                "hidden" => EOInt::from(self.hidden),
            },
        )
        .await?;

        tx.exec_drop(
            include_str!("./sql/update_stats.sql"),
            params! {
                "character_id" => self.id,
                "level" => self.level as u32,
                "experience" => self.experience,
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

        for item in &old_items {
            if !self.items.contains(item) {
                tx.exec_drop(
                    include_str!("./sql/delete_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                    },
                )
                .await?;
            }
        }

        for item in &self.items {
            if !old_items.contains(item) {
                tx.exec_drop(
                    include_str!("./sql/create_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            } else {
                tx.exec_drop(
                    include_str!("./sql/update_inventory_item.sql"),
                    params! {
                        "character_id" => self.id,
                        "item_id" => item.id,
                        "quantity" => item.amount,
                    },
                )
                .await?;
            }
        }

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
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
            base: CharacterBaseStats2 {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
        }
    }

    pub fn get_item_character_stats(&self) -> ItemCharacterStats {
        ItemCharacterStats {
            max_hp: self.max_hp,
            max_tp: self.max_tp,
            base: CharacterBaseStats {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
            secondary: CharacterSecondaryStats {
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
        }
    }
}
