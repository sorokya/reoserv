use eo::{pubs::EmfTileSpec, protocol::Coords};

use super::Map;

impl Map {
    pub fn is_tile_walkable(&self, coords: &Coords) -> bool {
        if let Some(row) = self.file.spec_rows.iter().find(|row| row.y == coords.y) {
            if let Some(tile) = row.tiles.iter().find(|tile| tile.x == coords.x) {
                return !matches!(
                    tile.spec,
                    EmfTileSpec::Wall
                        | EmfTileSpec::ChairDown
                        | EmfTileSpec::ChairLeft
                        | EmfTileSpec::ChairRight
                        | EmfTileSpec::ChairUp
                        | EmfTileSpec::ChairDownRight
                        | EmfTileSpec::ChairUpLeft
                        | EmfTileSpec::ChairAll
                        | EmfTileSpec::Chest
                        | EmfTileSpec::BankVault
                        | EmfTileSpec::MapEdge
                        | EmfTileSpec::Board1
                        | EmfTileSpec::Board2
                        | EmfTileSpec::Board3
                        | EmfTileSpec::Board4
                        | EmfTileSpec::Board5
                        | EmfTileSpec::Board6
                        | EmfTileSpec::Board7
                        | EmfTileSpec::Board8
                        | EmfTileSpec::Jukebox
                );
            }
        }

        // TODO: Ghost timer
        !self.is_tile_occupied(coords)
    }
}
