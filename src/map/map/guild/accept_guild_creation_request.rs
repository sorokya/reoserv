use super::super::Map;

impl Map {
    pub fn accept_guild_creation_request(&self, player_id: i32, invitee_player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let invitee_character = match self.characters.get(&invitee_player_id) {
            Some(character) => character,
            None => return,
        };

        if character.guild_tag.is_some() || invitee_character.guild_tag.is_some() {
            return;
        }

        invitee_character
            .player
            .as_ref()
            .unwrap()
            .add_guild_creation_player(player_id, character.name.clone());
    }
}
