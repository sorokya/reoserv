use std::cmp;

use eo::{
    data::{EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{server::item, PacketAction, PacketFamily, ShortItem},
};

use super::Map;

impl Map {
    pub fn junk_item(&mut self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        if amount == 0 {
            return;
        }

        let amount_to_junk = {
            let character = match self.characters.get(&target_player_id) {
                Some(character) => character,
                None => return,
            };

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
        let reply = item::Junk {
            junked_item: ShortItem {
                id: item_id,
                amount: amount_to_junk,
            },
            junked_item_amount: match character.items.iter().find(|i| i.id == item_id) {
                Some(item) => item.amount,
                None => 0,
            },
            weight: character.get_weight(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        let buf = builder.get();
        character
            .player
            .as_ref()
            .unwrap()
            .send(PacketAction::Junk, PacketFamily::Item, buf);
    }
}
