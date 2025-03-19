use eolib::protocol::{net::Item, r#pub::ItemType};
use eoplus::Arg;

use crate::{ITEM_DB, QUEST_DB};

use super::Character;

impl Character {
    pub fn unequip(&mut self, item_id: i32, sub_loc: i32) -> bool {
        if sub_loc > 1 {
            return false;
        }

        let item_record = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => return false,
        };

        match item_record.r#type {
            ItemType::Weapon => {
                if self.equipment.weapon != item_id {
                    return false;
                }
                self.equipment.weapon = 0;
            }
            ItemType::Shield => {
                if self.equipment.shield != item_id {
                    return false;
                }
                self.equipment.shield = 0;
            }
            ItemType::Armor => {
                if self.equipment.armor != item_id {
                    return false;
                }
                self.equipment.armor = 0;
            }
            ItemType::Hat => {
                if self.equipment.hat != item_id {
                    return false;
                }
                self.equipment.hat = 0;
            }
            ItemType::Boots => {
                if self.equipment.boots != item_id {
                    return false;
                }
                self.equipment.boots = 0;
            }
            ItemType::Gloves => {
                if self.equipment.gloves != item_id {
                    return false;
                }
                self.equipment.gloves = 0;
            }
            ItemType::Accessory => {
                if self.equipment.accessory != item_id {
                    return false;
                }
                self.equipment.accessory = 0;
            }
            ItemType::Belt => {
                if self.equipment.belt != item_id {
                    return false;
                }
                self.equipment.belt = 0;
            }
            ItemType::Necklace => {
                if self.equipment.necklace != item_id {
                    return false;
                }
                self.equipment.necklace = 0;
            }
            ItemType::Ring => {
                if self.equipment.ring[sub_loc as usize] != item_id {
                    return false;
                }
                self.equipment.ring[sub_loc as usize] = 0;
            }
            ItemType::Armlet => {
                if self.equipment.armlet[sub_loc as usize] != item_id {
                    return false;
                }
                self.equipment.armlet[sub_loc as usize] = 0;
            }
            ItemType::Bracer => {
                if self.equipment.bracer[sub_loc as usize] != item_id {
                    return false;
                }
                self.equipment.bracer[sub_loc as usize] = 0;
            }
            _ => {
                warn!(
                    "{} tried to unequip an invalid item type: {:?}",
                    self.name, item_record.r#type
                );
                return false;
            }
        }

        match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => {
                item.amount += 1;
            }
            None => {
                self.items.push(Item {
                    id: item_id,
                    amount: 1,
                });
            }
        }

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
                .find(|rule| rule.name == "UnequippedItem" && rule.args[0] == Arg::Int(item_id))
            {
                Some(rule) => rule,
                None => continue,
            };

            match quest
                .states
                .iter()
                .position(|state| state.name == rule.goto)
            {
                Some(next_state) => {
                    progress.state = next_state as i32;
                    quests_progressed.push(progress.id);
                }
                None => continue,
            };
        }

        for quest_id in quests_progressed {
            self.do_quest_actions(quest_id);
        }

        self.calculate_stats();
        true
    }
}
