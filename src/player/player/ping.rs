use eolib::{
    data::{EoSerialize, EoWriter},
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
            info!("player {} connection closed: ping timeout", self.id);
            false
        } else {
            self.bus.upcoming_sequence_start = generate_sequence_start();
            let mut writer = EoWriter::with_capacity(3);
            let sequence_bytes = get_ping_sequence_bytes(self.bus.upcoming_sequence_start);
            let packet = ConnectionPlayerServerPacket {
                seq1: sequence_bytes[0],
                seq2: sequence_bytes[1],
            };

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing ConnectionPlayerServerPacket: {}", e);
                return false;
            }

            self.bus.need_pong = true;
            self.bus
                .send(
                    PacketAction::Player,
                    PacketFamily::Connection,
                    writer.to_byte_array(),
                )
                .await
                .unwrap();
            true
        }
    }
}
