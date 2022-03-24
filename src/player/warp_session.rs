use eo::{
    data::EOShort,
    world::{TinyCoords, WarpAnimation},
};

pub struct WarpSession {
    pub map_id: EOShort,
    pub coords: TinyCoords,
    pub local: bool,
    pub animation: Option<WarpAnimation>,
}
