use eolib::protocol::net::Item;

use super::Character;

impl Character {
    pub fn add_bank_item(&mut self, item_id: i32, amount: i32) {
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
