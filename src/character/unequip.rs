use eolib::protocol::{net::Item, r#pub::ItemType};

use crate::ITEM_DB;

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
                if self.paperdoll.weapon != item_id {
                    return false;
                }
                self.paperdoll.weapon = 0;
            }
            ItemType::Shield => {
                if self.paperdoll.shield != item_id {
                    return false;
                }
                self.paperdoll.shield = 0;
            }
            ItemType::Armor => {
                if self.paperdoll.armor != item_id {
                    return false;
                }
                self.paperdoll.armor = 0;
            }
            ItemType::Hat => {
                if self.paperdoll.hat != item_id {
                    return false;
                }
                self.paperdoll.hat = 0;
            }
            ItemType::Boots => {
                if self.paperdoll.boots != item_id {
                    return false;
                }
                self.paperdoll.boots = 0;
            }
            ItemType::Gloves => {
                if self.paperdoll.gloves != item_id {
                    return false;
                }
                self.paperdoll.gloves = 0;
            }
            ItemType::Accessory => {
                if self.paperdoll.accessory != item_id {
                    return false;
                }
                self.paperdoll.accessory = 0;
            }
            ItemType::Belt => {
                if self.paperdoll.belt != item_id {
                    return false;
                }
                self.paperdoll.belt = 0;
            }
            ItemType::Necklace => {
                if self.paperdoll.necklace != item_id {
                    return false;
                }
                self.paperdoll.necklace = 0;
            }
            ItemType::Ring => {
                if self.paperdoll.ring[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.ring[sub_loc as usize] = 0;
            }
            ItemType::Armlet => {
                if self.paperdoll.armlet[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.armlet[sub_loc as usize] = 0;
            }
            ItemType::Bracer => {
                if self.paperdoll.bracer[sub_loc as usize] != item_id {
                    return false;
                }
                self.paperdoll.bracer[sub_loc as usize] = 0;
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

        self.calculate_stats();
        true
    }
}
