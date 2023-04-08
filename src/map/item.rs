use eo::{
    data::{EOInt, EOShort},
    protocol::{Coords, ItemMapInfo},
};

use crate::utils;

#[derive(Debug, Default)]
pub struct Item {
    pub id: EOShort,
    pub amount: EOInt,
    pub coords: Coords,
    pub owner: EOShort,
    // TODO: unprotect timer
}

impl Item {
    pub fn is_in_range(&self, coords: &Coords) -> bool {
        utils::in_range(&self.coords, coords)
    }

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
