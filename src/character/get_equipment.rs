use eolib::protocol::net::server::{EquipmentChange, EquipmentMapInfo, EquipmentWelcome};

use crate::ITEM_DB;

use super::Character;

impl Character {
    pub fn get_equipment_change(&self) -> EquipmentChange {
        EquipmentChange {
            boots: match self.equipment.boots {
                0 => 0,
                _ => match ITEM_DB.items.get(self.equipment.boots as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            armor: match self.equipment.armor {
                0 => 0,
                _ => match ITEM_DB.items.get(self.equipment.armor as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            hat: match self.equipment.hat {
                0 => 0,
                _ => match ITEM_DB.items.get(self.equipment.hat as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            weapon: match self.equipment.weapon {
                0 => 0,
                _ => match ITEM_DB.items.get(self.equipment.weapon as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
            shield: match self.equipment.shield {
                0 => 0,
                _ => match ITEM_DB.items.get(self.equipment.shield as usize - 1) {
                    Some(item) => item.spec1,
                    None => 0,
                },
            },
        }
    }

    pub fn get_equipment_map_info(&self) -> EquipmentMapInfo {
        let paperdoll = self.get_equipment_change();
        EquipmentMapInfo {
            boots: paperdoll.boots,
            armor: paperdoll.armor,
            hat: paperdoll.hat,
            weapon: paperdoll.weapon,
            shield: paperdoll.shield,
        }
    }

    pub fn get_equipment_welcome(&self) -> EquipmentWelcome {
        EquipmentWelcome {
            boots: self.equipment.boots,
            gloves: self.equipment.gloves,
            accessory: self.equipment.accessory,
            armor: self.equipment.armor,
            belt: self.equipment.belt,
            necklace: self.equipment.necklace,
            hat: self.equipment.hat,
            shield: self.equipment.shield,
            weapon: self.equipment.weapon,
            ring: self.equipment.ring,
            armlet: self.equipment.armlet,
            bracer: self.equipment.bracer,
        }
    }

    pub fn get_equipment_array(&self) -> [i32; 15] {
        [
            self.equipment.boots,
            self.equipment.accessory,
            self.equipment.gloves,
            self.equipment.belt,
            self.equipment.armor,
            self.equipment.necklace,
            self.equipment.hat,
            self.equipment.shield,
            self.equipment.weapon,
            self.equipment.ring[0],
            self.equipment.ring[1],
            self.equipment.armlet[0],
            self.equipment.armlet[1],
            self.equipment.bracer[0],
            self.equipment.bracer[1],
        ]
    }
}
