use eo::{protocol::Coords, pubs::EmfTileSpec};

use super::Map;

impl Map {
    pub fn get_tile(&self, coords: &Coords) -> Option<EmfTileSpec> {
        if let Some(row) = self.file.spec_rows.iter().find(|row| row.y == coords.y) {
            row.tiles
                .iter()
                .find(|tile| tile.x == coords.x)
                .map(|tile| tile.spec)
        } else {
            None
        }
    }
}
