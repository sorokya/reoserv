use eo::{
    data::{EOInt, i32},
    protocol::Coords,
};

#[derive(Debug)]
pub struct Door {
    pub coords: Coords,
    pub key: i32,
    pub open: bool,
    pub open_ticks: EOInt,
}

impl Door {
    pub fn new(coords: Coords, key: i32) -> Self {
        Self {
            coords,
            key,
            open: false,
            open_ticks: 0,
        }
    }
}
