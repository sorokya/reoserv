use eo::{data::EOChar, protocol::Coords};

pub fn is_in_bounds(coords: Coords, max_width: EOChar, max_height: EOChar) -> bool {
    coords.x <= max_width && coords.y <= max_height
}
