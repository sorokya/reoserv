use eolib::protocol::{map::MapWarp, Coords};

use super::super::Map;

impl Map {
    pub fn get_warp(&self, coords: &Coords) -> Option<MapWarp> {
        if let Some(row) = self.file.warp_rows.iter().find(|row| row.y == coords.y) {
            row.tiles
                .iter()
                .find(|tile| tile.x == coords.x)
                .map(|tile| tile.warp.to_owned())
        } else {
            None
        }
    }
}
