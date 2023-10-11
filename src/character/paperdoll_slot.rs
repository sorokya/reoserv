pub enum PaperdollSlot {
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

impl PaperdollSlot {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(PaperdollSlot::Boots),
            1 => Some(PaperdollSlot::Accessory),
            2 => Some(PaperdollSlot::Gloves),
            3 => Some(PaperdollSlot::Belt),
            4 => Some(PaperdollSlot::Armor),
            5 => Some(PaperdollSlot::Necklace),
            6 => Some(PaperdollSlot::Hat),
            7 => Some(PaperdollSlot::Shield),
            8 => Some(PaperdollSlot::Weapon),
            9 => Some(PaperdollSlot::Ring1),
            10 => Some(PaperdollSlot::Ring2),
            11 => Some(PaperdollSlot::Armlet1),
            12 => Some(PaperdollSlot::Armlet2),
            13 => Some(PaperdollSlot::Bracer1),
            14 => Some(PaperdollSlot::Bracer2),
            _ => None,
        }
    }
    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            PaperdollSlot::Boots
                | PaperdollSlot::Armor
                | PaperdollSlot::Hat
                | PaperdollSlot::Shield
                | PaperdollSlot::Weapon
        )
    }
}
