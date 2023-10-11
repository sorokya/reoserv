use eo::{
    data::{EOInt, EOShort},
    protocol::Item,
};

use super::Character;

impl Character {
    pub fn add_bank_item(&mut self, item_id: EOShort, amount: EOInt) {
        let existing_item = self.bank.iter_mut().find(|item| item.id == item_id);

        if let Some(existing_item) = existing_item {
            existing_item.amount += amount;
        } else {
            self.bank.push(Item {
                id: item_id,
                amount,
            });
        }
    }
}
