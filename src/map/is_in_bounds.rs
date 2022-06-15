use eo::{data::EOChar, world::TinyCoords};

pub fn is_in_bounds(coords: TinyCoords, max_width: EOChar, max_height: EOChar) -> bool {
    coords.x <= max_width && coords.y <= max_height
}
