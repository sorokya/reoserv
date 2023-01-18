use eo::protocol::{PaperdollB000a0hsw, PaperdollFull};

pub fn full_to_b000a0hsw(paperdoll: &PaperdollFull) -> PaperdollB000a0hsw {
    PaperdollB000a0hsw {
        boots: paperdoll.boots,
        armor: paperdoll.armor,
        hat: paperdoll.hat,
        shield: paperdoll.shield,
        weapon: paperdoll.weapon,
    }
}
