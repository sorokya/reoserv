use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{LockerAddClientPacket, LockerTakeClientPacket},
        Item, PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn locker_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match LockerAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing LockerAddClientPacket {}", e);
                    return;
                }
            };

            if self.trading {
                return;
            }

            map.add_locker_item(
                self.id,
                Item {
                    id: add.deposit_item.id,
                    amount: add.deposit_item.amount,
                },
            );
        }
    }

    fn locker_buy(&mut self) {
        if let Some(map) = &self.map {
            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.upgrade_locker(self.id, npc_index);
        }
    }

    fn locker_open(&mut self) {
        if let Some(map) = &self.map {
            map.open_locker(self.id);
        }
    }

    fn locker_take(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let take = match LockerTakeClientPacket::deserialize(&reader) {
                Ok(take) => take,
                Err(e) => {
                    error!("Error deserializing LockerTakeClientPacket {}", e);
                    return;
                }
            };

            map.take_locker_item(self.id, take.take_item_id);
        }
    }

    pub fn handle_locker(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with locker while trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Add => self.locker_add(reader),
            PacketAction::Buy => self.locker_buy(),
            PacketAction::Open => self.locker_open(),
            PacketAction::Take => self.locker_take(reader),
            _ => error!("Unhandled packet Locker_{:?}", action),
        }
    }
}
