use eo::{
    data::EOShort,
    protocol::{Coords, WarpAnimation},
};

pub struct WarpSession {
    pub map_id: EOShort,
    pub coords: Coords,
    pub local: bool,
    pub animation: Option<WarpAnimation>,
}
