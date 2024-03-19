use eolib::protocol::{
    net::{server::PaperdollPingServerPacket, PacketAction, PacketFamily},
    r#pub::ItemType,
};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn equip(&mut self, item_id: i32, sub_loc: i32) -> bool {
        if sub_loc > 1 {
            return false;
        }

        let existing_item = match self.items.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return false,
        };

        let item_record = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => return false,
        };

        if item_record.r#type == ItemType::Armor && item_record.spec2 != i32::from(self.gender) {
            return false;
        }

        if self.level < item_record.level_requirement
            || self.adj_strength < item_record.str_requirement
            || self.adj_intelligence < item_record.int_requirement
            || self.adj_wisdom < item_record.wis_requirement
            || self.adj_agility < item_record.agi_requirement
            || self.adj_constitution < item_record.con_requirement
            || self.adj_charisma < item_record.cha_requirement
        {
            return false;
        }

        if item_record.class_requirement != 0 && item_record.class_requirement != self.class {
            self.player.as_ref().unwrap().send(
                PacketAction::Ping,
                PacketFamily::Paperdoll,
                &PaperdollPingServerPacket {
                    class_id: self.class,
                },
            );

            return false;
        }

        match item_record.r#type {
            ItemType::Weapon => {
                if self.equipment.weapon != 0 {
                    return false;
                }
                self.equipment.weapon = item_id
            }
            ItemType::Shield => {
                if self.equipment.shield != 0 {
                    return false;
                }
                self.equipment.shield = item_id
            }
            ItemType::Armor => {
                if self.equipment.armor != 0 {
                    return false;
                }
                self.equipment.armor = item_id
            }
            ItemType::Hat => {
                if self.equipment.hat != 0 {
                    return false;
                }
                self.equipment.hat = item_id
            }
            ItemType::Boots => {
                if self.equipment.boots != 0 {
                    return false;
                }
                self.equipment.boots = item_id
            }
            ItemType::Gloves => {
                if self.equipment.gloves != 0 {
                    return false;
                }
                self.equipment.gloves = item_id
            }
            ItemType::Accessory => {
                if self.equipment.accessory != 0 {
                    return false;
                }
                self.equipment.accessory = item_id
            }
            ItemType::Belt => {
                if self.equipment.belt != 0 {
                    return false;
                }
                self.equipment.belt = item_id
            }
            ItemType::Necklace => {
                if self.equipment.necklace != 0 {
                    return false;
                }
                self.equipment.necklace = item_id
            }
            ItemType::Ring => {
                if self.equipment.ring[sub_loc as usize] != 0 {
                    return false;
                }
                self.equipment.ring[sub_loc as usize] = item_id
            }
            ItemType::Armlet => {
                if self.equipment.armlet[sub_loc as usize] != 0 {
                    return false;
                }
                self.equipment.armlet[sub_loc as usize] = item_id
            }
            ItemType::Bracer => {
                if self.equipment.bracer[sub_loc as usize] != 0 {
                    return false;
                }
                self.equipment.bracer[sub_loc as usize] = item_id
            }
            _ => {
                warn!(
                    "{} tried to equip an invalid item type: {:?}",
                    self.name, item_record.r#type
                );
                return false;
            }
        }

        if existing_item.amount <= 1 {
            self.items.retain(|item| item.id != item_id);
        } else {
            existing_item.amount -= 1;
        }

        self.calculate_stats();
        true
    }
}
