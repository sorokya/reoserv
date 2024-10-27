use eolib::protocol::{net::server::ItemMapInfo, Coords};

#[derive(Debug, Default)]
pub struct Item {
    pub id: i32,
    pub amount: i32,
    pub coords: Coords,
    pub owner: i32,
    pub protected_ticks: i32,
}

impl Item {
    pub fn to_map_info(&self, uid: &i32) -> ItemMapInfo {
        ItemMapInfo {
            uid: *uid,
            id: self.id,
            amount: self.amount,
            coords: self.coords,
        }
    }
}
