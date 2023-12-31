use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::ItemGetServerPacket, PacketAction, PacketFamily, ThreeItem},
};

use super::super::Map;

impl Map {
    pub fn give_item(&mut self, target_player_id: i32, item_id: i32, amount: i32) {
        if let Some(character) = self.characters.get_mut(&target_player_id) {
            character.add_item(item_id, amount);

            let reply = ItemGetServerPacket {
                taken_item_index: 0,
                taken_item: ThreeItem {
                    id: item_id,
                    amount,
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
                .send(PacketAction::Get, PacketFamily::Item, buf);
        }
    }
}
