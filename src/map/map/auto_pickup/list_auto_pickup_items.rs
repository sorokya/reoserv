use crate::ITEM_DB;

use super::super::Map;

impl Map {
    pub fn list_auto_pickup_items(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let item_db = ITEM_DB.load();
        let items = character
            .auto_pickup_items
            .iter()
            .filter_map(|item_id| {
                item_db
                    .items
                    .get(*item_id as usize - 1)
                    .map(|item| item.name.as_str())
            })
            .collect::<Vec<_>>();

        self.show_info_box(
            player_id,
            "Auto-Pickup Items:",
            if items.is_empty() {
                vec!["None"]
            } else {
                items
            },
        );
    }
}
