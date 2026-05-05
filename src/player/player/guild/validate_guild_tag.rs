use crate::SETTINGS;

pub fn validate_guild_tag(tag: &str) -> bool {
    tag.len() >= SETTINGS.load().guild.min_tag_length
        && tag.len() <= SETTINGS.load().guild.max_tag_length
        && tag.chars().all(|c| c.is_ascii_alphabetic())
}
