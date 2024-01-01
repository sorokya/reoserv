use eolib::protocol::{map::MapTileSpec, Coords};

use super::super::Map;

impl Map {
    pub fn get_tile(&self, coords: &Coords) -> Option<MapTileSpec> {
        if let Some(row) = self
            .file
            .tile_spec_rows
            .iter()
            .find(|row| row.y == coords.y)
        {
            row.tiles
                .iter()
                .find(|tile| tile.x == coords.x)
                .map(|tile| tile.tile_spec)
        } else {
            None
        }
    }
}
