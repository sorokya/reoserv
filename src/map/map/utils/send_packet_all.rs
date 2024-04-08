use bytes::Bytes;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn send_packet_all<T>(&self, action: PacketAction, family: PacketFamily, packet: T)
    where
        T: EoSerialize,
    {
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize packet: {}", e);
            return;
        }

        self.send_buf_all(action, family, writer.to_byte_array())
    }

    pub fn send_buf_all(&self, action: PacketAction, family: PacketFamily, buf: Bytes) {
        for player in self.characters.values().filter_map(|c| c.player.as_ref()) {
            player.send_buf(action, family, buf.clone());
        }
    }
}
