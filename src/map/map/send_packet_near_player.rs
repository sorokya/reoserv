use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::utils::in_range;

use super::Map;

impl Map {
    pub fn send_packet_near_player<T>(
        &self,
        player_id: EOShort,
        action: PacketAction,
        family: PacketFamily,
        packet: T,
    ) where
        T: Serializeable,
    {
        if let Some(target) = self.characters.get(&player_id) {
            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);
            let buf = builder.get();
            for (id, character) in self.characters.iter() {
                if id == &player_id || character.player.is_none() {
                    continue;
                }

                if in_range(&character.coords, &target.coords) {
                    character
                        .player
                        .as_ref()
                        .unwrap()
                        .send(action, family, buf.clone());
                }
            }
        }
    }
}
