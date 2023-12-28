use eolib::{protocol::net::{server::TalkServerServerPacket, PacketAction, PacketFamily}, data::{EoWriter, EoSerialize}};

use super::super::World;

impl World {
    pub fn broadcast_server_message(&self, message: &str) {
        let packet = TalkServerServerPacket {
            message: message.to_string(),
        };
        let mut writer = EoWriter::new();
        packet.serialize(&mut writer);
        let buf = writer.to_byte_array();
        for player in self.players.values() {
            player.send(PacketAction::Server, PacketFamily::Talk, buf.clone());
        }
    }
}
