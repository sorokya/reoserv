pub enum EquipmentSlot {
    Boots,
    Accessory,
    Gloves,
    Belt,
    Armor,
    Necklace,
    Hat,
    Shield,
    Weapon,
    Ring1,
    Ring2,
    Armlet1,
    Armlet2,
    Bracer1,
    Bracer2,
}

impl EquipmentSlot {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(EquipmentSlot::Boots),
            1 => Some(EquipmentSlot::Accessory),
            2 => Some(EquipmentSlot::Gloves),
            3 => Some(EquipmentSlot::Belt),
            4 => Some(EquipmentSlot::Armor),
            5 => Some(EquipmentSlot::Necklace),
            6 => Some(EquipmentSlot::Hat),
            7 => Some(EquipmentSlot::Shield),
            8 => Some(EquipmentSlot::Weapon),
            9 => Some(EquipmentSlot::Ring1),
            10 => Some(EquipmentSlot::Ring2),
            11 => Some(EquipmentSlot::Armlet1),
            12 => Some(EquipmentSlot::Armlet2),
            13 => Some(EquipmentSlot::Bracer1),
            14 => Some(EquipmentSlot::Bracer2),
            _ => None,
        }
    }
    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            EquipmentSlot::Boots
                | EquipmentSlot::Armor
                | EquipmentSlot::Hat
                | EquipmentSlot::Shield
                | EquipmentSlot::Weapon
        )
    }
}
