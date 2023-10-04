use std::cmp;

use chrono::{DateTime, Duration, Utc};
use eo::{
    data::{EOChar, EOInt, EOShort, EOThree},
    protocol::{Coords, Direction, NPCMapInfo},
};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{FORMULAS, NPC_DB};

#[derive(Debug, Default)]
pub struct Npc {
    pub id: EOShort,
    pub coords: Coords,
    pub direction: Direction,
    pub spawn_index: usize,
    pub alive: bool,
    pub dead_since: DateTime<Utc>,
    pub last_act: Option<DateTime<Utc>>,
    pub last_talk: Option<DateTime<Utc>>,
    pub walk_idle_for: Option<Duration>,
    pub hp: EOThree,
    pub max_hp: EOThree,
    pub opponents: Vec<NpcOpponent>,
}

#[derive(Debug, Default)]
pub struct NpcOpponent {
    pub player_id: EOShort,
    pub damage_dealt: EOInt,
    pub last_hit: DateTime<Utc>,
}

impl Npc {
    pub fn get_hp_percentage(&self) -> EOChar {
        let percent = (self.hp as f32 / self.max_hp as f32) * 100.0;
        percent.floor() as EOChar
    }

    pub fn to_map_info(&self, index: &EOChar) -> NPCMapInfo {
        NPCMapInfo {
            index: *index,
            id: self.id,
            coords: self.coords,
            direction: self.direction,
        }
    }

    pub fn damage(
        &mut self,
        player_id: EOShort,
        amount: u16,
        accuracy: u16,
        critical: bool,
    ) -> EOInt {
        let npc_data = match NPC_DB.npcs.get(self.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => {
                return 0;
            }
        };

        let context = match context_map! {
            "critical" => critical,
            "damage" => amount as f64,
            "target_armor" => npc_data.armor as f64,
            "target_sitting" => false,
            "accuracy" => accuracy as f64,
            "target_evade" => npc_data.evade as f64,
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

        if hit_rate < rand {
            return 0;
        }

        let damage = match eval_float_with_context(&FORMULAS.damage, &context) {
            Ok(amount) => cmp::min(amount.floor() as EOInt, self.hp as EOInt),
            Err(e) => {
                error!("Failed to calculate damage: {}", e);
                0
            }
        };

        self.hp -= damage as EOThree;
        if self.hp > 0 {
            match self.opponents.iter().position(|o| o.player_id == player_id) {
                Some(index) => {
                    let opponent = self.opponents.get_mut(index).unwrap();
                    opponent.damage_dealt += damage;
                    opponent.last_hit = Utc::now();
                }
                None => {
                    self.opponents.push(NpcOpponent {
                        player_id,
                        damage_dealt: damage,
                        last_hit: Utc::now(),
                    });
                }
            }
        } else {
            self.alive = false;
            self.dead_since = Utc::now();
            self.opponents.clear();
        }

        damage
    }
}

#[derive(Debug, Default)]
pub struct NPCBuilder {
    id: EOShort,
    coords: Coords,
    direction: Direction,
    spawn_index: usize,
    alive: bool,
    dead_since: DateTime<Utc>,
    last_act: Option<DateTime<Utc>>,
    last_talk: Option<DateTime<Utc>>,
    walk_idle_for: Option<Duration>,
    hp: EOThree,
    max_hp: EOThree,
}

impl NPCBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: EOShort) -> Self {
        self.id = id;
        self
    }

    pub fn coords(mut self, coords: Coords) -> Self {
        self.coords = coords;
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn spawn_index(mut self, spawn_index: usize) -> Self {
        self.spawn_index = spawn_index;
        self
    }

    pub fn alive(mut self, alive: bool) -> Self {
        self.alive = alive;
        self
    }

    pub fn dead_since(mut self, dead_since: DateTime<Utc>) -> Self {
        self.dead_since = dead_since;
        self
    }

    pub fn hp(mut self, hp: EOThree) -> Self {
        self.hp = hp;
        self
    }

    pub fn max_hp(mut self, max_hp: EOThree) -> Self {
        self.max_hp = max_hp;
        self
    }

    pub fn build(&self) -> Npc {
        Npc {
            id: self.id,
            coords: self.coords,
            direction: self.direction,
            spawn_index: self.spawn_index,
            alive: self.alive,
            dead_since: self.dead_since,
            last_act: self.last_act,
            last_talk: self.last_talk,
            walk_idle_for: self.walk_idle_for,
            hp: self.hp,
            max_hp: self.max_hp,
            opponents: Vec::new(),
        }
    }
}
