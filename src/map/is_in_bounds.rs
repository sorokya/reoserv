use eo::{world::Coords, data::EOShort};

pub fn is_in_bounds(coords: Coords, max_width: EOShort, max_height: EOShort) -> bool {
    coords.x >= 0 && coords.x < max_width && coords.y >= 0 && coords.y < max_height
}
