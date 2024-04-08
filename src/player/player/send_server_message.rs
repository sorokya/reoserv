use eolib::protocol::net::{server::TalkServerServerPacket, PacketAction, PacketFamily};

use super::Player;

impl Player {
    pub async fn send_server_message(&mut self, message: &str) {
        let _ = self
            .bus
            .send(
                PacketAction::Server,
                PacketFamily::Talk,
                TalkServerServerPacket {
                    message: message.to_string(),
                },
            )
            .await;
    }
}
