use super::{Character, PaperdollSlot};

impl Character {
    pub fn destroy_equipment(&mut self, slot: &PaperdollSlot) {
        match slot {
            PaperdollSlot::Boots => self.paperdoll.boots = 0,
            PaperdollSlot::Accessory => self.paperdoll.accessory = 0,
            PaperdollSlot::Gloves => self.paperdoll.gloves = 0,
            PaperdollSlot::Belt => self.paperdoll.belt = 0,
            PaperdollSlot::Armor => self.paperdoll.armor = 0,
            PaperdollSlot::Necklace => self.paperdoll.necklace = 0,
            PaperdollSlot::Hat => self.paperdoll.hat = 0,
            PaperdollSlot::Shield => self.paperdoll.shield = 0,
            PaperdollSlot::Weapon => self.paperdoll.weapon = 0,
            PaperdollSlot::Ring1 => self.paperdoll.ring[0] = 0,
            PaperdollSlot::Ring2 => self.paperdoll.ring[1] = 0,
            PaperdollSlot::Armlet1 => self.paperdoll.armlet[0] = 0,
            PaperdollSlot::Armlet2 => self.paperdoll.armlet[1] = 0,
            PaperdollSlot::Bracer1 => self.paperdoll.bracer[0] = 0,
            PaperdollSlot::Bracer2 => self.paperdoll.bracer[1] = 0,
        }
    }
}
