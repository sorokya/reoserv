use crate::SETTINGS;

pub fn validate_guild_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= SETTINGS.guild.max_name_length
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == ' ')
}
