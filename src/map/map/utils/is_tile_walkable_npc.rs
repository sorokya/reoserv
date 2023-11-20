use eo::{protocol::Coords, pubs::EmfTileSpec};

use super::super::Map;

impl Map {
    pub fn is_tile_walkable_npc(&self, coords: &Coords) -> bool {
        if let Some(row) = self.file.warp_rows.iter().find(|row| row.y == coords.y) {
            if row.tiles.iter().any(|warp| warp.x == coords.x) {
                return false;
            }
        }

        if let Some(tile) = self.get_tile(coords) {
            return !matches!(
                tile,
                EmfTileSpec::NPCBoundary
                    | EmfTileSpec::Wall
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

        true
    }
}
