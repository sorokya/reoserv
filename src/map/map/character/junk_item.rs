use std::cmp;

use eolib::{protocol::net::{server::ItemJunkServerPacket, ThreeItem, PacketAction, PacketFamily}, data::{EoWriter, EoSerialize}};

use super::super::Map;

impl Map {
    pub async fn junk_item(&mut self, target_player_id: i32, item_id: i32, amount: i32) {
        if amount == 0 {
            return;
        }

        let amount_to_junk = {
            let character = match self.characters.get(&target_player_id) {
                Some(character) => character,
                None => return,
            };

            if character.player.as_ref().unwrap().is_trading().await {
                return;
            }

            let actual_item = match character.items.iter().find(|i| i.id == item_id) {
                Some(item) => item,
                None => return,
            };

            cmp::min(amount, actual_item.amount)
        };

        {
            let character = self.characters.get_mut(&target_player_id).unwrap();
            character.remove_item(item_id, amount_to_junk);
        }

        let character = self.characters.get(&target_player_id).unwrap();
        let reply = ItemJunkServerPacket {
            junked_item: ThreeItem {
                id: item_id,
                amount: amount_to_junk,
            },
            remaining_amount: match character.items.iter().find(|i| i.id == item_id) {
                Some(item) => item.amount,
                None => 0,
            },
            weight: character.get_weight(),
        };

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);
        let buf = writer.to_byte_array();
        character
            .player
            .as_ref()
            .unwrap()
            .send(PacketAction::Junk, PacketFamily::Item, buf);
    }
}