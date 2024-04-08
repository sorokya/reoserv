use bytes::Bytes;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{PacketAction, PacketFamily},
        Coords,
    },
};

use crate::utils::in_range;

use super::super::Map;

impl Map {
    pub fn send_packet_near_exclude_player<T>(
        &self,
        coords: &Coords,
        exclude_player_id: i32,
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

        let buf = writer.to_byte_array();
        self.send_buf_near_exclude_player(coords, exclude_player_id, action, family, buf);
    }

    pub fn send_buf_near_exclude_player(
        &self,
        coords: &Coords,
        exclude_player_id: i32,
        action: PacketAction,
        family: PacketFamily,
        buf: Bytes,
    ) {
        for player in self.characters.iter().filter_map(|(id, c)| {
            if *id != exclude_player_id && in_range(&c.coords, coords) {
                c.player.as_ref()
            } else {
                None
            }
        }) {
            player.send_buf(action, family, buf.clone());
        }
    }
}
