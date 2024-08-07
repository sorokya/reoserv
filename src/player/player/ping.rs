use eolib::{
    packet::{generate_sequence_start, get_ping_sequence_bytes},
    protocol::net::{server::ConnectionPlayerServerPacket, PacketAction, PacketFamily},
};

use crate::player::ClientState;

use super::Player;

impl Player {
    pub async fn ping(&mut self) -> bool {
        if self.state == ClientState::Uninitialized {
            return true;
        }

        if self.bus.need_pong {
            self.close(format!(
                "player {} connection closed: ping timeout",
                self.id
            ))
            .await;
            false
        } else {
            self.bus.upcoming_sequence_start = generate_sequence_start();
            let sequence_bytes = get_ping_sequence_bytes(self.bus.upcoming_sequence_start);

            self.bus.need_pong = true;
            let _ = self
                .bus
                .send(
                    PacketAction::Player,
                    PacketFamily::Connection,
                    ConnectionPlayerServerPacket {
                        seq1: sequence_bytes[0],
                        seq2: sequence_bytes[1],
                    },
                )
                .await;

            true
        }
    }
}
