use bytes::Bytes;
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
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        self.send_buf_near_player(player_id, action, family, builder.get());
    }

    pub fn send_buf_near_player(
        &self,
        player_id: EOShort,
        action: PacketAction,
        family: PacketFamily,
        buf: Bytes,
    ) {
        if let Some(target) = self.characters.get(&player_id) {
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
