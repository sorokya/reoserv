use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkServerServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn broadcast_server_message(&self, message: &str) {
        let packet = TalkServerServerPacket {
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkServerServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            player.send(PacketAction::Server, PacketFamily::Talk, buf.clone());
        }
    }
}
