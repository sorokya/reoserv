use eolib::protocol::{net::server::WarpEffect, Coords};

pub struct WarpSession {
    pub map_id: i32,
    pub coords: Coords,
    pub local: bool,
    pub animation: Option<WarpEffect>,
}
