use std::collections::HashSet;

use eo::{
    world::TinyCoords,
};


pub fn is_occupied(
    coords: TinyCoords,
    occupied_tiles: &HashSet<TinyCoords>,
) -> bool {
    occupied_tiles.contains(&coords)
}
