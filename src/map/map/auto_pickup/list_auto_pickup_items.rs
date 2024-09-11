use crate::ITEM_DB;

use super::super::Map;

impl Map {
    pub fn list_auto_pickup_items(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let items = character
            .auto_pickup_items
            .iter()
            .filter_map(|item_id| match ITEM_DB.items.get(*item_id as usize - 1) {
                Some(item) => Some(item.name.as_str()),
                None => None,
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
