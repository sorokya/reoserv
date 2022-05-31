use eo::{
    data::{EOChar, EOShort},
    net::NpcMapInfo,
    world::{Direction, TinyCoords},
};

pub struct Npc {
    pub id: EOShort,
    pub coords: TinyCoords,
    pub direction: Direction,
}

impl Npc {
    pub fn new(id: EOShort, coords: TinyCoords, direction: Direction) -> Self {
        Self {
            id,
            coords,
            direction,
        }
    }

    pub fn to_npc_map_info(&self, index: &EOChar) -> NpcMapInfo {
        NpcMapInfo {
            index: *index,
            id: self.id,
            coords: self.coords,
            direction: self.direction,
        }
    }
}
