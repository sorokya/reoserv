use eo::{
    data::{EOInt, EOShort},
    net::ItemMapInfo,
    world::Coords,
};

pub struct Item {
    pub uid: EOShort,
    pub id: EOShort,
    pub amount: EOInt,
    pub coords: Coords,
    pub owner: EOShort,
    // TODO: unprotect timer
}

impl Item {
    pub fn new(uid: EOShort, id: EOShort, amount: EOInt, coords: Coords, owner: EOShort) -> Self {
        Self {
            uid,
            id,
            amount,
            coords,
            owner,
        }
    }

    pub fn to_item_map_info(&self) -> ItemMapInfo {
        ItemMapInfo {
            uid: self.uid,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
