use eo::data::EOShort;

use super::super::Map;

impl Map {
    pub async fn remove_trade_item(&mut self, player_id: EOShort, item_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.trade_items.iter().any(|item| item.id == item_id) {
            return;
        }

        character.trade_items.retain(|item| item.id != item_id);

        self.send_trade_update(player_id).await;
    }
}
