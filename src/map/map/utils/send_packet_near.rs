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
        packet.serialize(&mut writer);
        self.send_buf_near(coords, action, family, writer.to_byte_array())
    }

    pub fn send_buf_near(
        &self,
        coords: &Coords,
        action: PacketAction,
        family: PacketFamily,
        buf: Bytes,
    ) {
        for character in self.characters.values() {
            if in_range(coords, &character.coords) {
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .send(action, family, buf.clone());
            }
        }
    }
}
