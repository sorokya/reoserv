use eo::{
    data::{EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{server::item, PacketAction, PacketFamily, ShortItem},
};

use super::Map;

impl Map {
    pub fn give_item(&mut self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        if let Some(character) = self.characters.get_mut(&target_player_id) {
            character.add_item(item_id, amount);

            let reply = item::Get {
                taken_item_index: 0,
                taken_item: ShortItem {
                    id: item_id,
                    amount,
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
                .send(PacketAction::Get, PacketFamily::Item, buf);
        }
    }
}
