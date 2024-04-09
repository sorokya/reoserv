use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::NpcRangeRequestClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn npc_range_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match NpcRangeRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing NpcRangeRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_npcs(self.id, request.npc_indexes)
        }
    }

    pub fn handle_npc_range(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.npc_range_request(reader),
            _ => error!("Unhandled packet NPCRange_{:?}", action),
        }
    }
}
