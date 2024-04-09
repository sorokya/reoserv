use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{ChairRequestClientPacket, ChairRequestClientPacketSitActionData, SitAction},
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn chair_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match ChairRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing ChairRequestClientPacket {}", e);
                    return;
                }
            };

            match request.sit_action {
                SitAction::Sit => {
                    let coords = match request.sit_action_data {
                        Some(ChairRequestClientPacketSitActionData::Sit(sit)) => sit.coords,
                        _ => {
                            error!("Sit action data is not sit");
                            return;
                        }
                    };
                    map.sit_chair(self.id, coords);
                }
                SitAction::Stand => map.stand(self.id),
                _ => {}
            }
        }
    }

    pub fn handle_chair(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.chair_request(reader),
            _ => error!("Unhandled packet Chair_{:?}", action),
        }
    }
}
