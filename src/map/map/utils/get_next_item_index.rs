use eo::data::EOShort;

use super::super::Map;

impl Map {
    pub fn get_next_item_index(&self, seed: EOShort) -> EOShort {
        if self.items.iter().any(|(index, _)| *index == seed) {
            self.get_next_item_index(seed + 1)
        } else {
            seed
        }
    }
}
