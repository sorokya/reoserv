use std::cmp;

use eolib::protocol::net::{server::ItemKickServerPacket, Item, PacketAction, PacketFamily};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn lose_item(&mut self, player_id: i32, item_id: i32, amount: i32) {
        if item_id < 1 || amount <= 0 || amount > SETTINGS.limits.max_item {
            return;
        }

        let amount_to_junk = {
            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            let actual_item = match character.items.iter().find(|i| i.id == item_id) {
                Some(item) => item,
                None => return,
            };

            if let Some(player) = character.player.as_ref() {
                player.cancel_trade();
            }

            cmp::min(amount, actual_item.amount)
        };

        {
            let character = match self.characters.get_mut(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.remove_item(item_id, amount_to_junk);
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Kick,
                PacketFamily::Item,
                &ItemKickServerPacket {
                    item: Item {
                        id: item_id,
                        amount: character.get_item_amount(item_id),
                    },
                    current_weight: character.weight,
                },
            );
        }
    }
}
