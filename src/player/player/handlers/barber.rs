use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{BarberBuyClientPacket, BarberOpenClientPacket},
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn barber_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match BarberOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing BarberOpenClientPacket {}", e);
                    return;
                }
            };

            map.open_barber(self.id, open.npc_index, session_id);
        }
    }

    fn barber_buy(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let buy = match BarberBuyClientPacket::deserialize(&reader) {
                Ok(buy) => buy,
                Err(e) => {
                    error!("Error deserializing BarberBuyClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != buy.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.buy_haircut(self.id, npc_index, buy.hair_style, buy.hair_color);
        }
    }

    pub fn handle_barber(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with barber when trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Open => self.barber_open(reader),
            PacketAction::Buy => self.barber_buy(reader),
            _ => error!("Unhandled packet Barber_{:?}", action),
        }
    }
}
