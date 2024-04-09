use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{PartyAcceptClientPacket, PartyRemoveClientPacket, PartyRequestClientPacket},
        PacketAction, PartyRequestType,
    },
};

use crate::player::PartyRequest;

use super::super::Player;

impl Player {
    fn party_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match PartyRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing PartyRequestClientPacket {}", e);
                    return;
                }
            };

            match request.request_type {
                PartyRequestType::Join => {
                    map.party_request(request.player_id, PartyRequest::Join(self.id))
                }
                PartyRequestType::Invite => {
                    map.party_request(request.player_id, PartyRequest::Invite(self.id))
                }
                _ => {}
            }
        }
    }

    fn party_accept(&mut self, reader: EoReader) {
        let accept = match PartyAcceptClientPacket::deserialize(&reader) {
            Ok(accept) => accept,
            Err(e) => {
                error!("Error deserializing PartyAcceptClientPacket {}", e);
                return;
            }
        };

        self.world
            .accept_party_request(self.id, accept.inviter_player_id, accept.request_type);
    }

    fn party_remove(&mut self, reader: EoReader) {
        let remove = match PartyRemoveClientPacket::deserialize(&reader) {
            Ok(remove) => remove,
            Err(e) => {
                error!("Error deserializing PartyRemoveClientPacket {}", e);
                return;
            }
        };

        self.world.remove_party_member(self.id, remove.player_id);
    }

    fn party_take(&mut self) {
        self.world.request_party_list(self.id);
    }

    pub fn handle_party(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.party_request(reader),
            PacketAction::Accept => self.party_accept(reader),
            PacketAction::Remove => self.party_remove(reader),
            PacketAction::Take => self.party_take(),
            _ => error!("Unhandled packet Party_{:?}", action),
        }
    }
}
