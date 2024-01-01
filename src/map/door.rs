use eolib::protocol::Coords;

#[derive(Debug)]
pub struct Door {
    pub coords: Coords,
    pub key: i32,
    pub open: bool,
    pub open_ticks: i32,
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
