use std::cmp;

use eolib::protocol::{net::server::NpcMapInfo, Coords, Direction};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{FORMULAS, NPC_DB};

#[derive(Clone, Debug, Default)]
pub struct Npc {
    pub id: i32,
    pub coords: Coords,
    pub direction: Direction,
    pub spawn_type: i32,
    pub spawn_time: i32,
    pub spawn_index: Option<usize>,
    pub alive: bool,
    pub spawn_ticks: i32,
    pub act_ticks: i32,
    pub talk_ticks: i32,
    pub walk_idle_for: Option<i32>,
    pub hp: i32,
    pub max_hp: i32,
    pub opponents: Vec<NpcOpponent>,
    pub boss: bool,
    pub child: bool,
}

#[derive(Debug, Default, Clone)]
pub struct NpcOpponent {
    pub player_id: i32,
    pub damage_dealt: i32,
    pub bored_ticks: i32,
}

impl Npc {
    pub fn get_hp_percentage(&self) -> i32 {
        let percent = (self.hp as f32 / self.max_hp as f32) * 100.0;
        percent.floor() as i32
    }

    pub fn to_map_info(&self, index: &i32) -> NpcMapInfo {
        NpcMapInfo {
            index: *index,
            id: self.id,
            coords: self.coords,
            direction: self.direction,
        }
    }

    pub fn damage(&mut self, player_id: i32, amount: i32, accuracy: i32, critical: bool) -> i32 {
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
        if self.hp > 0 {
            match self.opponents.iter().position(|o| o.player_id == player_id) {
                Some(index) => {
                    let opponent = self.opponents.get_mut(index).unwrap();
                    opponent.damage_dealt += damage;
                    opponent.bored_ticks = 0;
                }
                None => {
                    self.opponents.push(NpcOpponent {
                        player_id,
                        damage_dealt: damage,
                        bored_ticks: 0,
                    });
                }
            }
        } else {
            self.alive = false;
            self.opponents.clear();

            if self.spawn_index.is_some() {
                self.spawn_ticks = self.spawn_time;
            }
        }

        damage
    }
}

#[derive(Debug, Default)]
pub struct NPCBuilder {
    id: i32,
    coords: Coords,
    direction: Direction,
    spawn_index: Option<usize>,
    spawn_type: i32,
    spawn_time: i32,
    alive: bool,
    spawn_ticks: i32,
    act_ticks: i32,
    talk_ticks: i32,
    walk_idle_for: Option<i32>,
    hp: i32,
    max_hp: i32,
    boss: bool,
    child: bool,
}

impl NPCBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: i32) -> Self {
        self.id = id;
        self
    }

    pub fn spawn_type(mut self, spawn_type: i32) -> Self {
        self.spawn_type = spawn_type;
        self
    }

    pub fn spawn_time(mut self, spawn_time: i32) -> Self {
        self.spawn_time = spawn_time;
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
        self.spawn_index = Some(spawn_index);
        self
    }

    pub fn alive(mut self, alive: bool) -> Self {
        self.alive = alive;
        self
    }

    pub fn spawn_ticks(mut self, spawn_ticks: i32) -> Self {
        self.spawn_ticks = spawn_ticks;
        self
    }

    pub fn hp(mut self, hp: i32) -> Self {
        self.hp = hp;
        self
    }

    pub fn max_hp(mut self, max_hp: i32) -> Self {
        self.max_hp = max_hp;
        self
    }

    pub fn boss(mut self, boss: bool) -> Self {
        self.boss = boss;
        self
    }

    pub fn child(mut self, child: bool) -> Self {
        self.child = child;
        self
    }

    pub fn build(&self) -> Npc {
        Npc {
            id: self.id,
            coords: self.coords,
            direction: self.direction,
            spawn_type: self.spawn_type,
            spawn_time: self.spawn_time,
            spawn_index: self.spawn_index,
            alive: self.alive,
            spawn_ticks: self.spawn_ticks,
            act_ticks: self.act_ticks,
            talk_ticks: self.talk_ticks,
            walk_idle_for: self.walk_idle_for,
            hp: self.hp,
            max_hp: self.max_hp,
            opponents: Vec::new(),
            boss: self.boss,
            child: self.child,
        }
    }
}
