use eo::data::{EOShort, EOChar};

use crate::character::Character;

pub fn set_character_stat(character: &mut Character, stat_name: String, value: EOShort) {
    match stat_name.as_str() {
        "level" => {
            character.level = value as EOChar;
        },
        _ => {}
    }
}