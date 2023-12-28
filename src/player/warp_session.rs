use eo::{
    data::i32,
    protocol::{Coords, WarpAnimation},
};

pub struct WarpSession {
    pub map_id: i32,
    pub coords: Coords,
    pub local: bool,
    pub animation: Option<WarpAnimation>,
}
