use eo::{data::{EOChar, EOShort}, world::{TinyCoords, Direction}, net::NpcMapInfo};

pub struct NPC {
    pub index: EOChar,
    pub id: EOShort,
    pub coords: TinyCoords,
    pub direction: Direction,
}

impl NPC {
    pub fn new(index: EOChar, id: EOShort, coords: TinyCoords, direction: Direction) -> Self {
        Self {
            index,
            id,
            coords,
            direction,
        }
    }

    pub fn to_npc_map_info(&self) -> NpcMapInfo {
        NpcMapInfo {
            index: self.index,
            id: self.id,
            coords: self.coords,
            direction: self.direction,
        }
    }
}