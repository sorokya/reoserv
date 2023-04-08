use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use eo::{
    data::{EOChar, EOShort, EOThree, EOInt},
    protocol::{Coords, Direction, NPCMapInfo},
};

use crate::utils;

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
    pub oppenents: HashMap<EOShort, EOInt>,
}

impl Npc {
    pub fn is_in_range(&self, coords: &Coords) -> bool {
        utils::in_range(
            &self.coords,
            coords,
        )
    }

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
            oppenents: HashMap::new(),
        }
    }
}
