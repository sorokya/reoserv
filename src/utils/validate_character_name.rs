pub fn validate_character_name(name: &str) -> bool {
    name.len() > 3 && name.len() <= 12 && name.chars().into_iter().all(|c| c.is_ascii_lowercase())
}
