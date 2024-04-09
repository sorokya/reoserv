use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::AttackUseClientPacket, PacketAction},
};

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

            map.attack(self.id, packet.direction, packet.timestamp);
        }
    }

    pub fn handle_attack(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Use => self.attack_use(reader),
            _ => error!("Unhandled packet Attack_{:?}", action),
        }
    }
}
