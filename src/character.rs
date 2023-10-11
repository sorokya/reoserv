use std::cmp;

use eo::{
    data::{EOChar, EOInt, EOShort, EOThree},
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
    pub evasion: EOShort, // TODO: rename to evade?
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

    pub fn get_weight(&self) -> Weight {
        Weight {
            current: cmp::min(self.weight, 250) as EOChar,
            max: self.max_weight as EOChar,
        }
    }

    pub fn get_icon(&self) -> PaperdollIcon {
        // TODO: group stuff

        match self.admin_level {
            AdminLevel::Player | AdminLevel::Spy | AdminLevel::LightGuide => PaperdollIcon::Player,
            AdminLevel::Guardian | AdminLevel::GameMaster => PaperdollIcon::Gm,
            AdminLevel::HighGameMaster => PaperdollIcon::Hgm,
        }
    }

    pub fn get_item_amount(&self, item_id: EOShort) -> EOInt {
        let existing_item = match self.items.iter().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return 0,
        };

        existing_item.amount
    }

    pub fn add_spell(&mut self, spell_id: EOShort) {
        if !self.has_spell(spell_id) {
            self.spells.push(Spell {
                id: spell_id,
                level: 1,
            });
        }
    }

    pub fn remove_spell(&mut self, spell_id: EOShort) {
        self.spells.retain(|spell| spell.id != spell_id);
    }

    pub fn has_spell(&self, spell_id: EOShort) -> bool {
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

    pub fn add_experience(&mut self, experience: EOThree) -> bool {
        self.experience += experience;

        let mut leveled_up = false;

        // TODO: Make this more accurate like official server
        // http://archive.today/brypq
        while self.experience > EXP_TABLE[self.level as usize + 1] {
            self.level += 1;
            self.stat_points += SETTINGS.world.stat_points_per_level as EOShort;
            self.skill_points += SETTINGS.world.skill_points_per_level as EOShort;
            leveled_up = true;
        }

        self.calculate_stats();
        leveled_up
    }
}
