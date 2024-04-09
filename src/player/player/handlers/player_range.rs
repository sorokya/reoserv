use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::PlayerRangeRequestClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn player_range_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match PlayerRangeRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing PlayerRangeRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_players(self.id, request.player_ids);
        }
    }

    pub fn handle_player_range(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.player_range_request(reader),
            _ => error!("Unhandled packet PlayerRange_{:?}", action),
        }
    }
}
