use std::cmp;

use eolib::protocol::net::{PacketAction, PacketFamily, ThreeItem, server::ItemObtainServerPacket};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn give_item(&mut self, player_id: i32, item_id: i32, amount: i32) {
        if let Some(character) = self.characters.get_mut(&player_id) {
            let amount = cmp::min(
                SETTINGS.load().limits.max_item - character.get_item_amount(item_id),
                amount,
            );

            character.add_item(item_id, amount);

            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Obtain,
                    PacketFamily::Item,
                    &ItemObtainServerPacket {
                        item: ThreeItem {
                            id: item_id,
                            amount,
                        },
                        current_weight: character.get_weight().current,
                    },
                );
            }
        }
    }
}
