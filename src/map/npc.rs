use chrono::{DateTime, Duration, Utc};
use eo::{
    data::{EOChar, EOShort, EOThree},
    protocol::{Coords, Direction, NPCMapInfo},
};

use crate::utils;

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
    // TODO: builder pattern?
    pub fn new(
        id: EOShort,
        coords: Coords,
        direction: Direction,
        spawn_index: usize,
        dead_since: DateTime<Utc>,
        last_act: DateTime<Utc>,
        does_talk: bool,
        last_talk: DateTime<Utc>,
        hp: EOThree,
    ) -> Self {
        Self {
            id,
            coords,
            direction,
            spawn_index,
            alive: false,
            dead_since,
            last_act,
            does_talk,
            last_talk,
            walk_idle_for: None,
            hp,
            max_hp: hp,
            target_player_id: None,
        }
    }

    pub fn is_in_range(&self, coords: Coords) -> bool {
        utils::in_range(
            self.coords.x.into(),
            self.coords.y.into(),
            coords.x.into(),
            coords.y.into(),
        )
    }

    pub fn is_in_range_distance(&self, coords: Coords, distance: f64) -> bool {
        utils::in_range_distance(
            self.coords.x.into(),
            self.coords.y.into(),
            coords.x.into(),
            coords.y.into(),
            distance,
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
