use std::cmp;

use crate::{ITEM_DB, SETTINGS};

use super::Character;

impl Character {
    pub fn can_hold(&self, item_id: i32, max_amount: i32) -> i32 {
        if self.weight > self.max_weight {
            return 0;
        }

        let item_data = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item_data) => item_data,
            None => return 0,
        };

        let remaining_weight = self.max_weight - self.weight;
        let max_items = if item_data.weight > 0 {
            (remaining_weight as f64 / item_data.weight as f64).floor() as i32
        } else {
            max_amount
        };

        let current_amount = self.get_item_amount(item_id);

        let amount = cmp::min(max_items, max_amount);
        cmp::min(SETTINGS.limits.max_item - current_amount, amount)
    }

    pub fn can_bank_hold(&self, item_id: i32, amount: i32) -> i32 {
        let item = match self.bank.iter().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return cmp::min(amount, SETTINGS.bank.max_item_amount - amount),
        };

        SETTINGS.bank.max_item_amount - item.amount
    }
}
