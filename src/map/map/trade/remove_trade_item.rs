use super::super::Map;

impl Map {
    pub async fn remove_trade_item(&mut self, player_id: i32, item_id: i32) {
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
