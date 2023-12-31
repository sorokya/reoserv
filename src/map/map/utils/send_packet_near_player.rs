use bytes::Bytes;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{PacketAction, PacketFamily},
};

use crate::utils::in_range;

use super::super::Map;

impl Map {
    pub fn send_packet_near_player<T>(
        &self,
        player_id: i32,
        action: PacketAction,
        family: PacketFamily,
        packet: T,
    ) where
        T: EoSerialize,
    {
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize packet: {}", e);
            return;
        }

        self.send_buf_near_player(player_id, action, family, writer.to_byte_array());
    }

    pub fn send_buf_near_player(
        &self,
        player_id: i32,
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
