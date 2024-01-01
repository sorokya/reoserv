use super::{Character, EquipmentSlot};

impl Character {
    pub fn destroy_equipment(&mut self, slot: &EquipmentSlot) {
        match slot {
            EquipmentSlot::Boots => self.equipment.boots = 0,
            EquipmentSlot::Accessory => self.equipment.accessory = 0,
            EquipmentSlot::Gloves => self.equipment.gloves = 0,
            EquipmentSlot::Belt => self.equipment.belt = 0,
            EquipmentSlot::Armor => self.equipment.armor = 0,
            EquipmentSlot::Necklace => self.equipment.necklace = 0,
            EquipmentSlot::Hat => self.equipment.hat = 0,
            EquipmentSlot::Shield => self.equipment.shield = 0,
            EquipmentSlot::Weapon => self.equipment.weapon = 0,
            EquipmentSlot::Ring1 => self.equipment.ring[0] = 0,
            EquipmentSlot::Ring2 => self.equipment.ring[1] = 0,
            EquipmentSlot::Armlet1 => self.equipment.armlet[0] = 0,
            EquipmentSlot::Armlet2 => self.equipment.armlet[1] = 0,
            EquipmentSlot::Bracer1 => self.equipment.bracer[0] = 0,
            EquipmentSlot::Bracer2 => self.equipment.bracer[1] = 0,
        }
    }
}
