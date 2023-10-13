use eo::protocol::Coords;

use super::Map;

impl Map {
    pub fn is_tile_occupied(&self, coords: &Coords) -> bool {
        self.characters
            .values()
            .any(|character| !character.hidden && character.coords == *coords)
            || self
                .npcs
                .values()
                .filter(|npc| npc.alive)
                .any(|npc| npc.coords == *coords)
    }
}
