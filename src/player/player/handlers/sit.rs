use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{SitAction, SitRequestClientPacket},
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn sit_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match SitRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing SitRequestClientPacket {}", e);
                    return;
                }
            };

            match request.sit_action {
                SitAction::Sit => map.sit(self.id),
                SitAction::Stand => map.stand(self.id),
                _ => {}
            }
        }
    }

    pub fn handle_sit(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.sit_request(reader),
            _ => error!("Unhandled packet Sit_{:?}", action),
        }
    }
}
