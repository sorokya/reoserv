use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            PaperdollAddClientPacket, PaperdollRemoveClientPacket, PaperdollRequestClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn paperdoll_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match PaperdollAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing PaperdollAddClientPacket {}", e);
                    return;
                }
            };

            map.equip(self.id, add.item_id, add.sub_loc);
        }
    }

    fn paperdoll_remove(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let remove = match PaperdollRemoveClientPacket::deserialize(&reader) {
                Ok(remove) => remove,
                Err(e) => {
                    error!("Error deserializing PaperdollRemoveClientPacket {}", e);
                    return;
                }
            };

            map.unequip(self.id, remove.item_id, remove.sub_loc);
        }
    }

    fn paperdoll_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match PaperdollRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing PaperdollRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_paperdoll(self.id, request.player_id);
        }
    }

    pub fn handle_paperdoll(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Add => self.paperdoll_add(reader),
            PacketAction::Remove => self.paperdoll_remove(reader),
            PacketAction::Request => self.paperdoll_request(reader),
            _ => error!("Unhandled packet Paperdoll_{:?}", action),
        }
    }
}
