use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        PacketAction,
        client::{MarriageOpenClientPacket, MarriageRequestClientPacket, MarriageRequestType},
    },
};

use crate::utils::validate_character_name;

use super::super::Player;

impl Player {
    fn marriage_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match MarriageOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing MarriageOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_law(self.id, open.npc_index, session_id);
        }
    }

    fn marriage_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match MarriageRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing MarriageRequestClientPacket {}", e);
                    return;
                }
            };

            let name = request.name.to_lowercase();
            if !validate_character_name(&name) {
                return;
            }

            match self.session_id {
                Some(session_id) => {
                    if session_id != request.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            match request.request_type {
                MarriageRequestType::MarriageApproval => {
                    map.request_marriage_approval(self.id, npc_index, name)
                }
                MarriageRequestType::Divorce => map.request_divorce(self.id, npc_index, name),
                _ => {}
            }
        }
    }

    pub fn handle_marriage(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Open => self.marriage_open(reader),
            PacketAction::Request => self.marriage_request(reader),
            _ => error!("Unhandled packet Marriage_{:?}", action),
        }
    }
}
