use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{ChestAddClientPacket, ChestOpenClientPacket, ChestTakeClientPacket},
        Item, PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn chest_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match ChestAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing ChestAddClientPacket {}", e);
                    return;
                }
            };

            map.add_chest_item(
                self.id,
                Item {
                    id: add.add_item.id,
                    amount: add.add_item.amount,
                },
            );
        }
    }

    fn chest_open(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let open = match ChestOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing ChestOpenClientPacket {}", e);
                    return;
                }
            };
            map.open_chest(self.id, open.coords);
        }
    }

    fn chest_take(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let take = match ChestTakeClientPacket::deserialize(&reader) {
                Ok(take) => take,
                Err(e) => {
                    error!("Error deserializing ChestTakeClientPacket {}", e);
                    return;
                }
            };

            map.take_chest_item(self.id, take.take_item_id);
        }
    }

    pub fn handle_chest(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Add => self.chest_add(reader),
            PacketAction::Open => self.chest_open(reader),
            PacketAction::Take => self.chest_take(reader),
            _ => error!("Unhandled packet Chest_{:?}", action),
        }
    }
}
