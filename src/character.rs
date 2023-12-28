use std::cmp;

use eo::{
    data::{i32, EOInt, i32, i32},
    protocol::{
        client::character::Create, AdminLevel, Coords, Direction, Gender, Item, PaperdollFull,
        PaperdollIcon, SitState, Skin, Spell, Weight,
    },
};

use chrono::prelude::*;
use mysql_async::Conn;

use crate::{player::PlayerHandle, EXP_TABLE, SETTINGS};

mod add_bank_item;
mod add_item;
mod calculate_stats;
mod can_hold;
mod create;
mod delete;
mod destroy_equipment;
mod equip;
mod get_paperdoll;
mod get_spawn_coords;
mod get_spawn_map;
mod get_stats;
mod load;
mod paperdoll_slot;
pub use paperdoll_slot::PaperdollSlot;
mod remove_item;
mod reset;
mod spell_state;
pub use spell_state::SpellState;
mod spell_target;
pub use spell_target::SpellTarget;
mod to_map_info;
mod unequip;
mod update;

#[derive(Debug, Clone, Default)]
pub struct Character {
    pub player_id: Option<i32>,
    pub player: Option<PlayerHandle>,
    pub id: EOInt,
    pub account_id: EOInt,
    pub name: String,
    pub title: Option<String>,
    pub home: String,
    pub fiance: Option<String>,
    pub partner: Option<String>,
    pub admin_level: AdminLevel,
    pub class: i32,
    pub gender: Gender,
    pub skin: Skin,
    pub hair_style: i32,
    pub hair_color: i32,
    pub bank_level: EOInt,
    pub gold_bank: EOInt,
    pub guild_name: Option<String>,
    pub guild_tag: Option<String>,
    pub guild_rank_id: Option<i32>,
    pub guild_rank_string: Option<String>,
    pub paperdoll: PaperdollFull,
    pub level: i32,
    pub experience: EOInt,
    pub hp: i32,
    pub max_hp: i32,
    pub tp: i32,
    pub max_tp: i32,
    pub max_sp: i32,
    pub weight: EOInt,
    pub max_weight: EOInt,
    pub base_strength: i32,
    pub base_intelligence: i32,
    pub base_wisdom: i32,
    pub base_agility: i32,
    pub base_constitution: i32,
    pub base_charisma: i32,
    pub adj_strength: i32,
    pub adj_intelligence: i32,
    pub adj_wisdom: i32,
    pub adj_agility: i32,
    pub adj_constitution: i32,
    pub adj_charisma: i32,
    pub stat_points: i32,
    pub skill_points: i32,
    pub karma: i32,
    pub usage: EOInt,
    pub min_damage: i32,
    pub max_damage: i32,
    pub accuracy: i32,
    pub evasion: i32,
    pub armor: i32,
    pub map_id: i32,
    pub coords: Coords,
    pub direction: Direction,
    pub sit_state: SitState,
    pub hidden: bool,
    pub items: Vec<Item>,
    pub bank: Vec<Item>,
    pub trade_items: Vec<Item>,
    pub spells: Vec<Spell>,
    pub logged_in_at: Option<DateTime<Utc>>,
    pub spell_state: SpellState,
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

    pub fn get_hp_percentage(&self) -> i32 {
        let percent = (self.hp as f32 / self.max_hp as f32) * 100.0;
        percent.floor() as i32
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let amount = cmp::min(amount, self.max_hp - self.hp);
        self.hp += amount;
        amount
    }

    pub fn tp_heal(&mut self, amount: i32) -> i32 {
        let amount = cmp::min(amount, self.max_tp - self.tp);
        self.tp += amount;
        amount
    }

    pub fn get_weight(&self) -> Weight {
        Weight {
            current: cmp::min(self.weight, 250) as i32,
            max: self.max_weight as i32,
        }
    }

    pub fn get_icon(&self, in_party: bool) -> PaperdollIcon {
        match self.admin_level {
            AdminLevel::Player | AdminLevel::Spy | AdminLevel::LightGuide => {
                if in_party {
                    PaperdollIcon::Party
                } else {
                    PaperdollIcon::Player
                }
            }
            AdminLevel::Guardian | AdminLevel::GameMaster => {
                if in_party {
                    PaperdollIcon::GmParty
                } else {
                    PaperdollIcon::Gm
                }
            }
            AdminLevel::HighGameMaster => {
                if in_party {
                    PaperdollIcon::HgmParty
                } else {
                    PaperdollIcon::Hgm
                }
            }
        }
    }

    pub fn get_item_amount(&self, item_id: i32) -> EOInt {
        match self.items.iter().find(|item| item.id == item_id) {
            Some(item) => item.amount,
            None => 0,
        }
    }

    pub fn get_bank_item_amount(&self, item_id: i32) -> EOInt {
        match self.bank.iter().find(|item| item.id == item_id) {
            Some(item) => item.amount,
            None => 0,
        }
    }

    pub fn add_spell(&mut self, spell_id: i32) {
        if !self.has_spell(spell_id) {
            self.spells.push(Spell {
                id: spell_id,
                level: 1,
            });
        }
    }

    pub fn remove_spell(&mut self, spell_id: i32) {
        self.spells.retain(|spell| spell.id != spell_id);
    }

    pub fn has_spell(&self, spell_id: i32) -> bool {
        self.spells.iter().any(|spell| spell.id == spell_id)
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

    pub fn add_experience(&mut self, experience: i32) -> bool {
        self.experience += experience;

        let mut leveled_up = false;

        // TODO: Make this more accurate like official server
        // http://archive.today/brypq
        while self.experience > EXP_TABLE[self.level as usize + 1] {
            self.level += 1;
            self.stat_points += SETTINGS.world.stat_points_per_level as i32;
            self.skill_points += SETTINGS.world.skill_points_per_level as i32;
            leveled_up = true;
        }

        self.calculate_stats();
        leveled_up
    }
}
