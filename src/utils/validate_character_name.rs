use crate::SETTINGS;

pub fn validate_character_name(name: &str) -> bool {
    name.len() >= SETTINGS.character.min_name_length
        && name.len() <= SETTINGS.character.max_name_length
        && name.chars().all(|c| c.is_ascii_lowercase())
}
