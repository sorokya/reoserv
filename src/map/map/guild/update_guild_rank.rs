use super::super::Map;

impl Map {
    pub fn update_guild_rank(&mut self, player_id: i32, rank: i32, rank_str: String) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.guild_rank = Some(rank);
        character.guild_rank_string = Some(rank_str);
    }
}
