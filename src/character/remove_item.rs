use eoplus::Arg;

use crate::{ITEM_DB, QUEST_DB};

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

        let total_amount = self.get_item_amount(item_id);

        let mut quests_progressed = Vec::new();
        for progress in self.quests.iter_mut() {
            let quest = match QUEST_DB.get(&progress.id) {
                Some(quest) => quest,
                None => continue,
            };

            let state = match quest.states.get(progress.state as usize) {
                Some(state) => state,
                None => continue,
            };

            let rule = match state
                .rules
                .iter()
                .find(|rule| rule.name == "LostItems" && rule.args[0] == Arg::Int(item_id))
            {
                Some(rule) => rule,
                None => continue,
            };

            let required_amount = match rule.args[1] {
                Arg::Int(amount) => amount,
                _ => continue,
            };

            if total_amount < required_amount {
                match quest
                    .states
                    .iter()
                    .position(|state| state.name == rule.goto)
                {
                    Some(next_state) => {
                        progress.state = next_state as i32;
                        quests_progressed.push(progress.id);
                    }
                    None => return,
                };
            }
        }

        for quest_id in quests_progressed {
            self.do_quest_actions(quest_id);
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
