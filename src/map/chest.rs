use chrono::{DateTime, Utc};
use eo::{
    data::{EOChar, EOShort, EOThree},
    protocol::Coords,
};

#[derive(Debug, Default)]
pub struct Chest {
    pub coords: Coords,
    pub key: Option<EOShort>,
    pub spawns: Vec<ChestSpawn>,
    pub items: Vec<ChestItem>,
}

#[derive(Debug, Default)]
pub struct ChestSpawn {
    pub slot: EOChar,
    pub item_id: EOShort,
    pub amount: EOThree,
    pub spawn_time: EOShort,
    pub last_taken: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct ChestItem {
    pub slot: EOChar,
    pub item_id: EOShort,
    pub amount: EOThree,
}
