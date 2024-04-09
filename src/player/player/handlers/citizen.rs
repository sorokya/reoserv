use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            CitizenAcceptClientPacket, CitizenOpenClientPacket, CitizenReplyClientPacket,
            CitizenRequestClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn citizen_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match CitizenOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing CitizenOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_inn(self.id, open.npc_index, session_id);
        }
    }

    fn citizen_reply(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let reply = match CitizenReplyClientPacket::deserialize(&reader) {
                Ok(reply) => reply,
                Err(e) => {
                    error!("Error deserializing CitizenReplyClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != reply.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.request_citizenship(self.id, npc_index, reply.answers);
        }
    }

    fn citizen_remove(&mut self) {
        if let Some(map) = &self.map {
            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.remove_citizenship(self.id, npc_index);
        }
    }

    fn citizen_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match CitizenRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing CitizenRequestClientPacket {}", e);
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

            map.request_sleep(self.id, npc_index);
        }
    }

    fn citizen_accept(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let accept = match CitizenAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing CitizenAcceptClientPacket {}", e);
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

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            let cost = match self.sleep_cost {
                Some(cost) => cost,
                None => return,
            };

            map.sleep(self.id, npc_index, cost);
        }
    }

    pub fn handle_citizen(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with citizen npc while trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Open => self.citizen_open(reader),
            PacketAction::Reply => self.citizen_reply(reader),
            PacketAction::Remove => self.citizen_remove(),
            PacketAction::Request => self.citizen_request(reader),
            PacketAction::Accept => self.citizen_accept(reader),
            _ => error!("Unhandled packet Citizen_{:?}", action),
        }
    }
}
