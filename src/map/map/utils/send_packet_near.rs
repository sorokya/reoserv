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
    pub fn send_packet_near<T>(
        &self,
        coords: &Coords,
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

        self.send_buf_near(coords, action, family, writer.to_byte_array())
    }

    pub fn send_buf_near(
        &self,
        coords: &Coords,
        action: PacketAction,
        family: PacketFamily,
        buf: Bytes,
    ) {
        for player in self.characters.values().filter_map(|c| {
            if in_range(&c.coords, coords) {
                c.player.as_ref()
            } else {
                None
            }
        }) {
            player.send_buf(action, family, buf.clone());
        }
    }
}
