use crate::SETTINGS;

pub fn validate_guild_rank(rank: &str) -> bool {
    rank.len() <= SETTINGS.guild.max_rank_length
        && rank.chars().all(|c| c.is_ascii_alphabetic() || c == ' ')
}
