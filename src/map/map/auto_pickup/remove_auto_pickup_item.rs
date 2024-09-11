use crate::ITEM_DB;

use super::super::Map;

impl Map {
    pub fn remove_auto_pickup_item(&mut self, player_id: i32, item_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let item_name = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item.name.as_str(),
            None => return,
        };

        character.auto_pickup_items.retain(|id| *id != item_id);

        if let Some(player) = character.player.as_ref() {
            player.send_server_message(&format!("Auto-Pickup Item Removed: {}", item_name));
        }
    }
}
