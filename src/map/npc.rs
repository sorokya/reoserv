use chrono::{DateTime, Utc, Duration};
use eo::{
    data::{EOChar, EOShort},
    net::NpcMapInfo,
    world::{Direction, TinyCoords},
};

use crate::utils;

pub struct Npc {
    pub id: EOShort,
    pub coords: TinyCoords,
    pub direction: Direction,
    pub spawn_index: usize,
    pub alive: bool,
    pub dead_since: DateTime<Utc>,
    pub last_act: DateTime<Utc>,
    pub last_talk: DateTime<Utc>,
    pub walk_idle_for: Option<Duration>,
}

impl Npc {
    pub fn new(id: EOShort, coords: TinyCoords, direction: Direction, spawn_index: usize, dead_since: DateTime<Utc>, last_act: DateTime<Utc>, last_talk: DateTime<Utc>) -> Self {
        Self {
            id,
            coords,
            direction,
            spawn_index,
            alive: false,
            dead_since,
            last_act,
            last_talk,
            walk_idle_for: None,
        }
    }

    pub fn is_in_range(&self, coords: TinyCoords) -> bool {
        utils::in_range(
            self.coords.x.into(),
            self.coords.y.into(),
            coords.x.into(),
            coords.y.into(),
        )
    }

    pub fn is_in_range_distance(&self, coords: TinyCoords, distance: f64) -> bool {
        utils::in_range_distance(
            self.coords.x.into(),
            self.coords.y.into(),
            coords.x.into(),
            coords.y.into(),
            distance,
        )
    }

    pub fn to_map_info(&self, index: &EOChar) -> NpcMapInfo {
        NpcMapInfo {
            index: *index,
            id: self.id,
            coords: self.coords,
            direction: self.direction,
        }
    }
}
