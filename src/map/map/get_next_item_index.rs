use eo::data::EOShort;

use super::Map;

impl Map {
    pub fn get_next_item_index(&self, seed: EOShort) -> EOShort {
        if self.items.iter().any(|item| item.index == seed) {
            self.get_next_item_index(seed + 1)
        } else {
            seed
        }
    }
}
