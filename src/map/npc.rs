use eo::{
    data::{EOChar, EOShort},
    net::NpcMapInfo,
    world::{Direction, TinyCoords},
};

pub struct Npc {
    pub index: EOChar,
    pub id: EOShort,
    pub coords: TinyCoords,
    pub direction: Direction,
}

impl Npc {
    pub fn _new(index: EOChar, id: EOShort, coords: TinyCoords, direction: Direction) -> Self {
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
