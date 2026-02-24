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
    ) -> anyhow::Result<i32> {
        self.item_index_counter += 1;
        if self.item_index_counter > 64_000 {
            self.item_index_counter = 1;
        }

        if self
            .items
            .iter()
            .any(|item| item.index == self.item_index_counter)
        {
            return Err(anyhow::anyhow!("Item index collision"));
        }

        self.items.push(Item {
            index: self.item_index_counter,
            id,
            amount,
            coords,
            owner,
            protected_ticks,
        });
        Ok(self.item_index_counter)
    }
}
