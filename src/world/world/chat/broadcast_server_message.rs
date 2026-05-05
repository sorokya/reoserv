use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{PacketAction, PacketFamily, server::TalkServerServerPacket},
};

use super::super::World;

impl World {
    pub fn broadcast_server_message(&self, message: &str) {
        let packet = TalkServerServerPacket {
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            tracing::error!("Failed to serialize TalkServerServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            player.send_buf(PacketAction::Server, PacketFamily::Talk, buf.clone());
        }
    }
}
