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
        packet: &T,
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
            for player in self.characters.iter().filter_map(|(id, c)| {
                if *id != player_id && in_range(&c.coords, &target.coords) {
                    c.player.as_ref()
                } else {
                    None
                }
            }) {
                player.send_buf(action, family, buf.clone());
            }
        }
    }
}
