use std::cmp;

use eo::{data::EOChar, protocol::Coords};

use super::super::Map;

impl Map {
    pub fn get_adjacent_tiles(&self, coords: &Coords) -> Vec<Coords> {
        let mut adjacent_tiles = Vec::with_capacity(5);
        adjacent_tiles.push(*coords);
        adjacent_tiles.push(Coords {
            x: coords.x,
            y: cmp::max(coords.y as i32 - 1, 0) as EOChar,
        });
        adjacent_tiles.push(Coords {
            x: coords.x,
            y: cmp::min(coords.y as i32 + 1, self.file.height as i32) as EOChar,
        });
        adjacent_tiles.push(Coords {
            x: cmp::max(coords.x as i32 - 1, 0) as EOChar,
            y: coords.y,
        });
        adjacent_tiles.push(Coords {
            x: cmp::min(coords.x as i32 + 1, self.file.width as i32) as EOChar,
            y: coords.y,
        });

        adjacent_tiles.dedup();
        adjacent_tiles
    }
}
