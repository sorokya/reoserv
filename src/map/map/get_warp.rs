use eo::{protocol::Coords, pubs::EmfWarp};

use super::Map;

impl Map {
    pub fn get_warp(&self, coords: &Coords) -> Option<EmfWarp> {
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
