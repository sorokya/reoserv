use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            PriestAcceptClientPacket, PriestOpenClientPacket, PriestRequestClientPacket,
            PriestUseClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn priest_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match PriestOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing PriestOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_priest(self.id, open.npc_index, session_id);
        }
    }

    fn priest_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match PriestRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing PriestRequestClientPacket {}", e);
                    return;
                }
            };

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

            map.request_wedding(self.id, npc_index, request.name);
        }
    }

    fn priest_accept(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let accept = match PriestAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing PriestAcceptClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != accept.session_id {
                        return;
                    }
                }
                None => return,
            }

            map.accept_wedding_request(self.id);
        }
    }

    fn priest_use(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let r#use = match PriestUseClientPacket::deserialize(&reader) {
                Ok(r#use) => r#use,
                Err(e) => {
                    error!("Error deserializing PriestUseClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != r#use.session_id {
                        return;
                    }
                }
                None => return,
            }

            map.say_i_do(self.id);
        }
    }

    pub fn handle_priest(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Open => self.priest_open(reader),
            PacketAction::Request => self.priest_request(reader),
            PacketAction::Accept => self.priest_accept(reader),
            PacketAction::Use => self.priest_use(reader),
            _ => error!("Unhandled packet Priest_{:?}", action),
        }
    }
}
