use eo::{
    data::{EOInt, i32},
    protocol::Item,
};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn add_item(&mut self, item_id: i32, amount: EOInt) {
        let existing_item = self.items.iter_mut().find(|item| item.id == item_id);

        if let Some(existing_item) = existing_item {
            existing_item.amount += amount;
        } else {
            self.items.push(Item {
                id: item_id,
                amount,
            });
        }

        if let Some(item) = ITEM_DB.items.get(item_id as usize - 1) {
            self.weight += item.weight as EOInt * amount;
        }
    }
}
