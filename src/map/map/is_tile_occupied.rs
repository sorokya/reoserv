use eo::protocol::Coords;

use super::Map;

impl Map {
    pub fn is_tile_occupied(&self, coords: &Coords) -> bool {
        self.characters
            .values()
            .any(|character| character.coords == *coords)
            || self.npcs.values().any(|npc| npc.coords == *coords)
    }
}
