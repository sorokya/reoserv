use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::BookRequestClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn book_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match BookRequestClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing BookRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_book(self.id, request.player_id);
        }
    }

    pub fn handle_book(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.book_request(reader),
            _ => error!("Unhandled packet Book_{:?}", action),
        }
    }
}
