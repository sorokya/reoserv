use eo::{data::EOShort, protocol::Coords, pubs::EmfTileSpec};

use crate::utils::in_client_range;

use super::Map;

impl Map {
    pub fn player_in_range_of_tile(&self, player_id: EOShort, spec_id: EmfTileSpec) -> bool {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return false,
        };

        for row in &self.file.spec_rows {
            for tile in row.tiles.iter().filter(|tile| tile.spec == spec_id) {
                if in_client_range(
                    &character.coords,
                    &Coords {
                        x: tile.x,
                        y: row.y,
                    },
                ) {
                    return true;
                }
            }
        }

        false
    }
}
