use eo::{
    data::{EOInt, EOShort},
    protocol::{Coords, ItemMapInfo},
};

#[derive(Debug, Default)]
pub struct Item {
    pub id: EOShort,
    pub amount: EOInt,
    pub coords: Coords,
    pub owner: EOShort,
    // TODO: unprotect timer
}

impl Item {
    // TODO: Implement as a trait
    pub fn to_item_map_info(&self, uid: EOShort) -> ItemMapInfo {
        ItemMapInfo {
            uid,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
