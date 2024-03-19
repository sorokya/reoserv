use std::cmp;

use eolib::protocol::net::{server::ItemGetServerPacket, PacketAction, PacketFamily, ThreeItem};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn give_item(&mut self, target_player_id: i32, item_id: i32, amount: i32) {
        if let Some(character) = self.characters.get_mut(&target_player_id) {
            let amount = cmp::min(
                SETTINGS.limits.max_item - character.get_item_amount(item_id),
                amount,
            );

            character.add_item(item_id, amount);

            character.player.as_ref().unwrap().send(
                PacketAction::Get,
                PacketFamily::Item,
                &ItemGetServerPacket {
                    taken_item_index: 0,
                    taken_item: ThreeItem {
                        id: item_id,
                        amount,
                    },
                    weight: character.get_weight(),
                },
            );
        }
    }
}
