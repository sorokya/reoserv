use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            ShopBuyClientPacket, ShopCreateClientPacket, ShopOpenClientPacket, ShopSellClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn shop_buy(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let buy = match ShopBuyClientPacket::deserialize(&reader) {
                Ok(buy) => buy,
                Err(e) => {
                    error!("Error deserializing ShopBuyClientPacket {}", e);
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

            map.buy_item(self.id, npc_index, buy.buy_item);
        }
    }

    fn shop_create(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let create = match ShopCreateClientPacket::deserialize(&reader) {
                Ok(create) => create,
                Err(e) => {
                    error!("Error deserializing ShopCreateClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != create.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.craft_item(self.id, npc_index, create.craft_item_id);
        }
    }

    fn shop_open(&mut self, reader: EoReader) {
        let session_id = self.generate_session_id();

        if let Some(map) = &self.map {
            let open = match ShopOpenClientPacket::deserialize(&reader) {
                Ok(open) => open,
                Err(e) => {
                    error!("Error deserializing ShopCreateClientPacket {}", e);
                    return;
                }
            };

            map.open_shop(self.id, open.npc_index, session_id);
        }
    }

    fn shop_sell(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let sell = match ShopSellClientPacket::deserialize(&reader) {
                Ok(sell) => sell,
                Err(e) => {
                    error!("Error deserializing ShopSellClientPacket {}", e);
                    return;
                }
            };

            match self.session_id {
                Some(session_id) => {
                    if session_id != sell.session_id {
                        return;
                    }
                }
                None => return,
            }

            let npc_index = match self.interact_npc_index {
                Some(npc_index) => npc_index,
                None => return,
            };

            map.sell_item(self.id, npc_index, sell.sell_item);
        }
    }

    pub fn handle_shop(&mut self, action: PacketAction, reader: EoReader) {
        // Prevent interacting with shop when trading
        if self.trading {
            return;
        }

        match action {
            PacketAction::Buy => self.shop_buy(reader),
            PacketAction::Create => self.shop_create(reader),
            PacketAction::Open => self.shop_open(reader),
            PacketAction::Sell => self.shop_sell(reader),
            _ => error!("Unhandled packet Shop_{:?}", action),
        }
    }
}
