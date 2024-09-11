use super::super::Map;

impl Map {
    pub fn clear_auto_pickup_items(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.auto_pickup_items.clear();

        if let Some(player) = character.player.as_ref() {
            player.send_server_message("Auto-Pickup Items Cleared");
        }
    }
}
