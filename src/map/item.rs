use eolib::protocol::{Coords, net::server::ItemMapInfo};

#[derive(Debug, Default)]
pub struct Item {
    pub index: i32,
    pub id: i32,
    pub amount: i32,
    pub coords: Coords,
    pub owner: i32,
    pub protected_ticks: i32,
}

impl Item {
    pub fn to_map_info(&self) -> ItemMapInfo {
        ItemMapInfo {
            uid: self.index,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
