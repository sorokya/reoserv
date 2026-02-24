use eolib::protocol::Coords;

use crate::map::Item;

use super::super::Map;

impl Map {
    pub fn add_item(
        &mut self,
        id: i32,
        amount: i32,
        coords: Coords,
        owner: i32,
        protected_ticks: i32,
    ) -> i32 {
        let item_index = self.get_next_item_index(1);
        self.items.push(Item {
            index: item_index,
            id,
            amount,
            coords,
            owner,
            protected_ticks,
        });
        item_index
    }
}
