use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::client::WalkPlayerClientPacket,
};

use super::super::Player;

impl Player {
    pub fn handle_walk(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let packet = match WalkPlayerClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing WalkPlayerClientPacket {}", e);
                    return;
                }
            };

            map.walk(
                self.id,
                packet.walk_action.direction,
                packet.walk_action.coords,
                packet.walk_action.timestamp,
            );
        }
    }
}
