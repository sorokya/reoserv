use chrono::{DateTime, Duration, Utc};
use eo::{
    data::{EOChar, EOShort, EOThree},
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
    pub last_act: DateTime<Utc>,
    pub does_talk: bool,
    pub last_talk: DateTime<Utc>,
    pub walk_idle_for: Option<Duration>,
    pub hp: EOThree,
    pub max_hp: EOThree,
    pub target_player_id: Option<EOShort>,
}

impl Npc {
    pub fn is_in_range(&self, coords: &Coords) -> bool {
        utils::in_range(
            &self.coords,
            coords,
        )
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
    last_act: DateTime<Utc>,
    does_talk: bool,
    last_talk: DateTime<Utc>,
    walk_idle_for: Option<Duration>,
    hp: EOThree,
    max_hp: EOThree,
    target_player_id: Option<EOShort>,
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

    pub fn last_act(mut self, last_act: DateTime<Utc>) -> Self {
        self.last_act = last_act;
        self
    }

    pub fn does_talk(mut self, does_talk: bool) -> Self {
        self.does_talk = does_talk;
        self
    }

    pub fn last_talk(mut self, last_talk: DateTime<Utc>) -> Self {
        self.last_talk = last_talk;
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
            does_talk: self.does_talk,
            last_talk: self.last_talk,
            walk_idle_for: self.walk_idle_for,
            hp: self.hp,
            max_hp: self.max_hp,
            target_player_id: self.target_player_id,
        }
    }
}
