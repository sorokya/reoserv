use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::RangeRequestClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn range_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match RangeRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing RangeRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_players_and_npcs(self.id, request.player_ids, request.npc_indexes);
        }
    }

    pub fn handle_range(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.range_request(reader),
            _ => error!("Unhandled packet Range_{:?}", action),
        }
    }
}
