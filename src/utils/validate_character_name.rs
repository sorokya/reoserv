use crate::SETTINGS;

pub fn validate_character_name(name: &str) -> bool {
    name.len() >= SETTINGS.load().character.min_name_length
        && name.len() <= SETTINGS.load().character.max_name_length
        && name.chars().all(|c| c.is_ascii_lowercase())
}
