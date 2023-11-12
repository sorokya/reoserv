use eo::{data::EOShort, protocol::Item, pubs::EifItemSpecial};

use crate::ITEM_DB;

use super::Map;

impl Map {
    pub async fn add_trade_item(&mut self, player_id: EOShort, item: Item) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.get_item_amount(item.id) < item.amount {
            return;
        }

        let offered = character.trade_items.iter().any(|i| i.id == item.id);

        if !offered && character.trade_items.len() >= 10 {
            return;
        }

        let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
            Some(item_data) => item_data,
            None => return,
        };

        if item_data.special == EifItemSpecial::Lore {
            return;
        }

        if offered {
            let mut trade_item = character
                .trade_items
                .iter_mut()
                .find(|i| i.id == item.id)
                .unwrap();

            trade_item.amount = item.amount;
        } else {
            character.trade_items.push(item);
        }

        self.send_trade_update(player_id).await;
    }
}
