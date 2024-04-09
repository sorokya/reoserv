use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{BankAddClientPacket, BankOpenClientPacket, BankTakeClientPacket},
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn bank_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match BankAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing BankAddClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != add.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.deposit_gold(self.id, npc_index, add.amount);
        }
    }

    fn bank_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match BankOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing BankOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_bank(self.id, open.npc_index, session_id);
        }
    }

    fn bank_take(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let take = match BankTakeClientPacket::deserialize(&reader) {
                Ok(take) => take,
                Err(e) => {
                    error!("Error deserializing BankTakeClientPacket {}", e);
                    return;
                }
            };

            match &self.session_id {
                Some(session_id) => {
                    if *session_id != take.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match &self.interact_npc_index {
                Some(npc_index) => *npc_index,
                None => return,
            };

            map.withdraw_gold(self.id, npc_index, take.amount);
        }
    }

    pub fn handle_bank(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with bank when trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Add => self.bank_add(reader),
            PacketAction::Open => self.bank_open(reader),
            PacketAction::Take => self.bank_take(reader),
            _ => error!("Unhandled packet Bank_{:?}", action),
        }
    }
}
