use eolib::protocol::Gender;

use crate::{character::Character, SETTINGS};

pub fn dressed_for_wedding(character: &Character) -> bool {
    match character.gender {
        Gender::Female => character.equipment.armor == SETTINGS.marriage.female_armor_id,
        Gender::Male => character.equipment.armor == SETTINGS.marriage.male_armor_id,
        _ => false,
    }
}
