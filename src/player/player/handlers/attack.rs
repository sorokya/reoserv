use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::AttackUseClientPacket, PacketAction},
};

use crate::utils::timestamp_diff;

use super::super::Player;

impl Player {
    fn attack_use(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let packet = match AttackUseClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing AttackUseClientPacket {}", e);
                    return;
                }
            };

            if timestamp_diff(packet.timestamp, self.timestamp) < 48 {
                return;
            }

            self.timestamp = packet.timestamp;

            map.attack(self.id, packet.direction);
        }
    }

    pub fn handle_attack(&mut self, action: PacketAction, reader: EoReader) {
        if self.captcha.is_some() {
            return;
        }

        match action {
            PacketAction::Use => self.attack_use(reader),
            _ => error!("Unhandled packet Attack_{:?}", action),
        }
    }
}
