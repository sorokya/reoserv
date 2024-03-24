use chrono::prelude::*;
use eolib::protocol::{
    net::{
        client::CharacterCreateClientPacket,
        server::{CharacterIcon, EquipmentPaperdoll, SitState},
        Item, Spell, Weight,
    },
    AdminLevel, Coords, Direction, Gender,
};
use eoplus::Arg;
use evalexpr::{context_map, eval_float_with_context};
use mysql_async::Conn;
use rand::Rng;
use std::cmp;

use crate::{player::PlayerHandle, EXP_TABLE, FORMULAS, QUEST_DB, SETTINGS};

mod add_bank_item;
mod add_item;
mod calculate_stats;
mod can_hold;
mod create;
mod delete;
mod destroy_equipment;
mod equip;
mod equipment_slot;
mod get_equipment;
mod get_spawn_coords;
mod get_spawn_map;
mod get_stats;
mod load;
pub use equipment_slot::EquipmentSlot;
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
    pub id: i32,
    pub account_id: i32,
    pub name: String,
    pub title: Option<String>,
    pub home: String,
    pub fiance: Option<String>,
    pub partner: Option<String>,
    pub admin_level: AdminLevel,
    pub class: i32,
    pub gender: Gender,
    pub skin: i32,
    pub hair_style: i32,
    pub hair_color: i32,
    pub bank_level: i32,
    pub gold_bank: i32,
    pub guild_name: Option<String>,
    pub guild_tag: Option<String>,
    pub guild_rank: Option<i32>,
    pub guild_rank_string: Option<String>,
    pub equipment: EquipmentPaperdoll,
    pub level: i32,
    pub experience: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub tp: i32,
    pub max_tp: i32,
    pub max_sp: i32,
    pub weight: i32,
    pub max_weight: i32,
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
    pub usage: i32,
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
    pub quests: Vec<QuestProgress>,
}

#[derive(Clone, Debug, Default)]
pub struct QuestProgress {
    pub id: i32,
    pub state: i32,
    pub npc_kills: Vec<(i32, i32)>,
    pub player_kills: i32,
}

impl QuestProgress {
    pub fn add_npc_kill(&mut self, npc_id: i32) {
        match self.npc_kills.iter_mut().find(|(id, _)| npc_id == *id) {
            Some((_, kills)) => *kills += 1,
            None => self.npc_kills.push((npc_id, 1)),
        }
    }

    pub fn get_npc_kills(&self, npc_id: i32) -> i32 {
        match self.npc_kills.iter().find(|(id, _)| npc_id == *id) {
            Some((_, kills)) => *kills,
            None => 0,
        }
    }
}

impl Character {
    pub fn from_creation(account_id: i32, create: &CharacterCreateClientPacket) -> Self {
        Character {
            account_id,
            gender: create.gender,
            hair_style: create.hair_style,
            hair_color: create.hair_color,
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

    pub fn damage(&mut self, amount: i32, accuracy: i32, critical: bool) -> i32 {
        let context = match context_map! {
            "critical" => critical,
            "damage" => amount as f64,
            "target_armor" => self.armor as f64,
            "target_sitting" => false,
            "accuracy" => accuracy as f64,
            "target_evade" => self.evasion as f64,
        } {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to generate formula context: {}", e);
                return 0;
            }
        };

        let hit_rate = match eval_float_with_context(&FORMULAS.hit_rate, &context) {
            Ok(hit_rate) => hit_rate,
            Err(e) => {
                error!("Failed to calculate hit rate: {}", e);
                0.0
            }
        };

        let mut rng = rand::thread_rng();
        let rand = rng.gen_range(0.0..1.0);

        let damage = if hit_rate < rand {
            0
        } else {
            match eval_float_with_context(&FORMULAS.damage, &context) {
                Ok(amount) => amount.floor() as i32,
                Err(e) => {
                    error!("Failed to calculate damage: {}", e);
                    0
                }
            }
        };

        self.hp -= cmp::min(damage, self.hp) as i32;

        damage
    }

    pub fn get_weight(&self) -> Weight {
        Weight {
            current: cmp::min(self.weight, 250),
            max: self.max_weight,
        }
    }

    pub fn get_icon(&self, in_party: bool) -> CharacterIcon {
        match self.admin_level {
            AdminLevel::Guardian | AdminLevel::GameMaster => {
                if in_party {
                    CharacterIcon::GmParty
                } else {
                    CharacterIcon::Gm
                }
            }
            AdminLevel::HighGameMaster => {
                if in_party {
                    CharacterIcon::HgmParty
                } else {
                    CharacterIcon::Hgm
                }
            }
            _ => {
                if in_party {
                    CharacterIcon::Party
                } else {
                    CharacterIcon::Player
                }
            }
        }
    }

    pub fn get_item_amount(&self, item_id: i32) -> i32 {
        match self.items.iter().find(|item| item.id == item_id) {
            Some(item) => item.amount,
            None => 0,
        }
    }

    pub fn get_bank_item_amount(&self, item_id: i32) -> i32 {
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

    pub fn is_guild_leader(&self) -> bool {
        self.guild_rank == Some(1)
    }

    pub fn get_quest_progress(&self, quest_id: i32) -> QuestProgress {
        match self.quests.iter().find(|q| q.id == quest_id) {
            Some(progress) => progress.to_owned(),
            None => QuestProgress {
                id: quest_id,
                ..Default::default()
            },
        }
    }

    pub fn save_quest_progress(&mut self, quest_id: i32, state: i32) {
        match self.quests.iter_mut().find(|q| q.id == quest_id) {
            Some(progress) => progress.state = state,
            None => self.quests.push(QuestProgress {
                id: quest_id,
                state,
                ..Default::default()
            }),
        }
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
            self.stat_points += SETTINGS.world.stat_points_per_level;
            self.skill_points += SETTINGS.world.skill_points_per_level;
            leveled_up = true;
        }

        self.calculate_stats();
        leveled_up
    }

    pub fn talked_to_npc(&mut self, behavior_id: i32) {
        let mut updated_quests: Vec<i32> = Vec::new();
        for progress in self.quests.iter_mut() {
            let quest = match QUEST_DB.get(&progress.id) {
                Some(quest) => quest,
                None => continue,
            };

            let state = match quest.states.get(progress.state as usize) {
                Some(state) => state,
                None => continue,
            };

            let rule =
                match state.rules.iter().find(|rule| {
                    rule.name == "TalkedToNpc" && rule.args[0] == Arg::Int(behavior_id)
                }) {
                    Some(rule) => rule,
                    None => continue,
                };

            match quest
                .states
                .iter()
                .position(|state| state.name == rule.goto)
            {
                Some(next_state) => {
                    progress.state = next_state as i32;
                    updated_quests.push(progress.id);
                }
                None => continue,
            };
        }

        for quest_id in updated_quests {
            self.do_quest_actions(quest_id);
        }
    }

    fn do_quest_actions(&self, quest_id: i32) {
        let state = match self.quests.iter().find(|progress| progress.id == quest_id) {
            Some(progress) => progress.state,
            None => return,
        };

        let state = match QUEST_DB.get(&quest_id) {
            Some(quest) => match quest.states.get(state as usize) {
                Some(state) => state,
                None => return,
            },
            None => return,
        };

        let player = match self.player {
            Some(ref player) => player,
            None => return,
        };

        for action in state.actions.iter() {
            match action.name.as_str() {
                "AddNpcText" | "AddNpcChat" | "AddNpcInput" => {}
                _ => player.quest_action(action.name.to_owned(), action.args.to_owned()),
            }
        }
    }
}
