use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{server::paperdoll, PacketAction, PacketFamily},
    pubs::EifItemType,
};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn equip(&mut self, item_id: EOShort, sub_loc: EOChar) -> bool {
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

        if item_record.r#type == EifItemType::Armor && item_record.spec2 != self.gender.to_char() {
            return false;
        }

        if (self.level as EOShort) < item_record.level_req
            || self.adj_strength < item_record.str_req
            || self.adj_intelligence < item_record.int_req
            || self.adj_wisdom < item_record.wis_req
            || self.adj_agility < item_record.agi_req
            || self.adj_constitution < item_record.con_req
            || self.adj_charisma < item_record.cha_req
        {
            return false;
        }

        if item_record.class_req != 0 && item_record.class_req != self.class as EOShort {
            let reply = paperdoll::Ping {
                class_id: self.class,
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            self.player.as_ref().unwrap().send(
                PacketAction::Ping,
                PacketFamily::Paperdoll,
                builder.get(),
            );
            return false;
        }

        match item_record.r#type {
            EifItemType::Weapon => {
                if self.paperdoll.weapon != 0 {
                    return false;
                }
                self.paperdoll.weapon = item_id
            }
            EifItemType::Shield => {
                if self.paperdoll.shield != 0 {
                    return false;
                }
                self.paperdoll.shield = item_id
            }
            EifItemType::Armor => {
                if self.paperdoll.armor != 0 {
                    return false;
                }
                self.paperdoll.armor = item_id
            }
            EifItemType::Hat => {
                if self.paperdoll.hat != 0 {
                    return false;
                }
                self.paperdoll.hat = item_id
            }
            EifItemType::Boots => {
                if self.paperdoll.boots != 0 {
                    return false;
                }
                self.paperdoll.boots = item_id
            }
            EifItemType::Gloves => {
                if self.paperdoll.gloves != 0 {
                    return false;
                }
                self.paperdoll.gloves = item_id
            }
            EifItemType::Accessory => {
                if self.paperdoll.accessory != 0 {
                    return false;
                }
                self.paperdoll.accessory = item_id
            }
            EifItemType::Belt => {
                if self.paperdoll.belt != 0 {
                    return false;
                }
                self.paperdoll.belt = item_id
            }
            EifItemType::Necklace => {
                if self.paperdoll.necklace != 0 {
                    return false;
                }
                self.paperdoll.necklace = item_id
            }
            EifItemType::Ring => {
                if self.paperdoll.ring[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.ring[sub_loc as usize] = item_id
            }
            EifItemType::Armlet => {
                if self.paperdoll.armlet[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.armlet[sub_loc as usize] = item_id
            }
            EifItemType::Bracer => {
                if self.paperdoll.bracer[sub_loc as usize] != 0 {
                    return false;
                }
                self.paperdoll.bracer[sub_loc as usize] = item_id
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
