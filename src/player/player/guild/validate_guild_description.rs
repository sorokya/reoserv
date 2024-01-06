use crate::SETTINGS;

const ALLOWED_SYMBOLS: [char; 5] = [' ', '@', '-', '_', '.'];

pub fn validate_guild_description(description: &str) -> bool {
    description.len() <= SETTINGS.guild.max_description_length
        && description
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ALLOWED_SYMBOLS.contains(&c))
}
