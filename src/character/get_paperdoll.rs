use eolib::protocol::net::server::{EquipmentChange, EquipmentMapInfo};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn get_paperdoll_bahws(&self) -> EquipmentChange {
        EquipmentChange {
            boots: match self.paperdoll.boots {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.boots as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            armor: match self.paperdoll.armor {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.armor as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            hat: match self.paperdoll.hat {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.hat as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            weapon: match self.paperdoll.weapon {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.weapon as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            shield: match self.paperdoll.shield {
                0 => 0,
                _ => match ITEM_DB.items.get(self.paperdoll.shield as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
        }
    }

    pub fn get_paperdoll_b000a0hsw(&self) -> EquipmentMapInfo {
        let paperdoll = self.get_paperdoll_bahws();
        EquipmentMapInfo {
            boots: paperdoll.boots,
            armor: paperdoll.armor,
            hat: paperdoll.hat,
            weapon: paperdoll.weapon,
            shield: paperdoll.shield,
        }
    }

    pub fn get_paperdoll_array(&self) -> [i32; 15] {
        [
            self.paperdoll.boots,
            self.paperdoll.accessory,
            self.paperdoll.gloves,
            self.paperdoll.belt,
            self.paperdoll.armor,
            self.paperdoll.necklace,
            self.paperdoll.hat,
            self.paperdoll.shield,
            self.paperdoll.weapon,
            self.paperdoll.ring[0],
            self.paperdoll.ring[1],
            self.paperdoll.armlet[0],
            self.paperdoll.armlet[1],
            self.paperdoll.bracer[0],
            self.paperdoll.bracer[1],
        ]
    }
}
