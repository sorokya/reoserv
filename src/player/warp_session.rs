use eo::{data::EOShort, world::TinyCoords};

pub struct WarpSession {
    pub map_id: EOShort,
    pub coords: TinyCoords,
    pub local: bool,
}
