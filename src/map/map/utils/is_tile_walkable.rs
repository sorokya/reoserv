use eolib::protocol::{map::MapTileSpec, Coords};

use super::super::Map;

impl Map {
    pub fn is_tile_walkable(&self, coords: &Coords) -> bool {
        if let Some(tile) = self.get_tile(coords) {
            return !matches!(
                tile,
                MapTileSpec::Wall
                    | MapTileSpec::ChairDown
                    | MapTileSpec::ChairLeft
                    | MapTileSpec::ChairRight
                    | MapTileSpec::ChairUp
                    | MapTileSpec::ChairDownRight
                    | MapTileSpec::ChairUpLeft
                    | MapTileSpec::ChairAll
                    | MapTileSpec::Chest
                    | MapTileSpec::BankVault
                    | MapTileSpec::Edge
                    | MapTileSpec::Board1
                    | MapTileSpec::Board2
                    | MapTileSpec::Board3
                    | MapTileSpec::Board4
                    | MapTileSpec::Board5
                    | MapTileSpec::Board6
                    | MapTileSpec::Board7
                    | MapTileSpec::Board8
                    | MapTileSpec::Jukebox
            );
        }

        true
    }
}
