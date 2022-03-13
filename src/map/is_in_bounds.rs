use eo::{data::EOShort, world::Coords};

pub fn is_in_bounds(coords: Coords, max_width: EOShort, max_height: EOShort) -> bool {
    coords.x <= max_width && coords.y <= max_height
}
