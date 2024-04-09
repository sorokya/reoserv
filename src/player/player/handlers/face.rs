use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::FacePlayerClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn face_player(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let packet = match FacePlayerClientPacket::deserialize(&reader) {
                Ok(packet) => packet,
                Err(e) => {
                    error!("Error deserializing FacePlayerClientPacket {}", e);
                    return;
                }
            };

            map.face(self.id, packet.direction);
        }
    }

    pub fn handle_face(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Player => self.face_player(reader),
            _ => error!("Unhandled packet Face_{:?}", action),
        }
    }
}
