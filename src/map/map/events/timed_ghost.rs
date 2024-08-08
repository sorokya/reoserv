use std::cmp;

use super::super::Map;

impl Map {
    pub fn timed_ghost(&mut self) {
        for character in self.characters.values_mut() {
            character.ghost_ticks = cmp::max(0, character.ghost_ticks - 1);
        }
    }
}
