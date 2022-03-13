use eo::{data::EOShort, world::TinyCoords};

pub struct WarpSession {
    pub id: EOShort,
    pub map_id: EOShort,
    pub coords: TinyCoords,
    pub local: bool,
}
