use eo::{
    data::{EOInt, EOShort},
    protocol::Coords,
};

#[derive(Debug)]
pub struct Door {
    pub coords: Coords,
    pub key: EOShort,
    pub open: bool,
    pub open_ticks: EOInt,
}

impl Door {
    pub fn new(coords: Coords, key: EOShort) -> Self {
        Self {
            coords,
            key,
            open: false,
            open_ticks: 0,
        }
    }
}
