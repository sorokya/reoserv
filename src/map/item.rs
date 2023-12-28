use eo::{
    data::{EOInt, i32},
    protocol::{Coords, ItemMapInfo},
};

#[derive(Debug, Default)]
pub struct Item {
    pub id: i32,
    pub amount: EOInt,
    pub coords: Coords,
    pub owner: i32,
    // TODO: unprotect timer
}

impl Item {
    // TODO: Implement as a trait
    pub fn to_item_map_info(&self, uid: i32) -> ItemMapInfo {
        ItemMapInfo {
            uid,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
