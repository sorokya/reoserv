use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::client::WalkPlayerClientPacket,
};

use super::super::Player;

impl Player {
    pub fn handle_walk(&mut self, reader: EoReader) {
        if self.captcha.is_some() {
            return;
        }

        if let Some(map) = &self.map {
            let packet = match WalkPlayerClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing WalkPlayerClientPacket {}", e);
                    return;
                }
            };

            if packet.walk_action.timestamp - self.timestamp < 36 {
                debug!(
                    "Walk action too fast: {} - {} == {}",
                    packet.walk_action.timestamp,
                    self.timestamp,
                    packet.walk_action.timestamp - self.timestamp
                );
                return;
            }

            self.timestamp = packet.walk_action.timestamp;

            map.walk(
                self.id,
                packet.walk_action.direction,
                packet.walk_action.coords,
            );
        }
    }
}
