use eolib::protocol::net::{server::MessagePongServerPacket, PacketAction, PacketFamily};

use super::super::Player;

impl Player {
    async fn message_ping(&mut self) {
        let _ = self
            .bus
            .send(
                PacketAction::Pong,
                PacketFamily::Message,
                MessagePongServerPacket::new(),
            )
            .await;
    }

    pub async fn handle_message(&mut self, action: PacketAction) {
        match action {
            PacketAction::Ping => self.message_ping().await,
            _ => error!("Unhandled packet Message_{:?}", action),
        }
    }
}
