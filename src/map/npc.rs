use chrono::{DateTime, Utc};
use eo::{
    data::{EOChar, EOShort},
    net::NpcMapInfo,
    world::{Direction, TinyCoords},
};

pub struct Npc {
    pub id: EOShort,
    pub coords: TinyCoords,
    pub direction: Direction,
    pub spawn_index: usize,
    pub alive: bool,
    pub dead_since: DateTime<Utc>,
}

impl Npc {
    pub fn new(id: EOShort, coords: TinyCoords, direction: Direction, spawn_index: usize, dead_since: DateTime<Utc>) -> Self {
        Self {
            id,
            coords,
            direction,
            spawn_index,
            alive: false,
            dead_since,
        }
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
