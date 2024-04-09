use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::DoorOpenClientPacket, PacketAction},
};

use super::super::Player;

impl Player {
    fn door_open(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let open = match DoorOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing DoorOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_door(self.id, open.coords);
        }
    }

    pub fn handle_door(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Open => self.door_open(reader),
            _ => error!("Unhandled packet Door_{:?}", action),
        }
    }
}
