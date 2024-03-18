use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkServerServerPacket, PacketAction, PacketFamily},
};

use super::Player;

impl Player {
    pub async fn send_server_message(&mut self, message: &str) {
        let packet = TalkServerServerPacket {
            message: message.to_string(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing TalkServerServerPacket: {}", e);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Server,
                PacketFamily::Talk,
                writer.to_byte_array(),
            )
            .await;
    }
}
