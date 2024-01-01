use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn remove_item(&mut self, item_id: i32, amount: i32) {
        let existing_item = match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return,
        };

        if existing_item.amount <= amount {
            self.items.retain(|item| item.id != item_id);
        } else {
            existing_item.amount -= amount;
        }

        if let Some(item) = ITEM_DB.items.get(item_id as usize - 1) {
            self.weight -= item.weight * amount;
        }
    }

    pub fn remove_bank_item(&mut self, item_id: i32, amount: i32) {
        let existing_item = match self.bank.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return,
        };

        if existing_item.amount <= amount {
            self.bank.retain(|item| item.id != item_id);
        } else {
            existing_item.amount -= amount;
        }
    }
}
