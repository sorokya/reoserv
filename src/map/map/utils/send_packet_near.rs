use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{Coords, PacketAction, PacketFamily},
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
        T: Serializeable,
    {
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        self.send_buf_near(coords, action, family, builder.get())
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
