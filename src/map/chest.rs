use chrono::{DateTime, Utc};
use eo::{
    data::{i32, i32, i32},
    protocol::Coords,
};

#[derive(Debug, Default)]
pub struct Chest {
    pub coords: Coords,
    pub key: Option<i32>,
    pub spawns: Vec<ChestSpawn>,
    pub items: Vec<ChestItem>,
}

#[derive(Debug, Default)]
pub struct ChestSpawn {
    pub slot: i32,
    pub item_id: i32,
    pub amount: i32,
    pub spawn_time: i32,
    pub last_taken: DateTime<Utc>,
}

#[derive(Debug, Default)]
pub struct ChestItem {
    pub slot: i32,
    pub item_id: i32,
    pub amount: i32,
}
