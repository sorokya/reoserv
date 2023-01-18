use eo::{
    data::{EOInt, EOShort},
    protocol::{Coords, ItemMapInfo},
};

use crate::utils;

pub struct Item {
    pub uid: EOShort,
    pub id: EOShort,
    pub amount: EOInt,
    pub coords: Coords,
    pub owner: EOShort,
    // TODO: unprotect timer
}

impl Item {
    pub fn _new(uid: EOShort, id: EOShort, amount: EOInt, coords: Coords, owner: EOShort) -> Self {
        Self {
            uid,
            id,
            amount,
            coords,
            owner,
        }
    }

    pub fn _is_in_range(&self, coords: Coords) -> bool {
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

    // TODO: Implement as a trait
    pub fn to_item_map_info(&self) -> ItemMapInfo {
        ItemMapInfo {
            uid: self.uid,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
