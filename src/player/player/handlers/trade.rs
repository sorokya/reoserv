use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            TradeAcceptClientPacket, TradeAddClientPacket, TradeAgreeClientPacket,
            TradeRemoveClientPacket, TradeRequestClientPacket,
        },
        PacketAction,
    },
};

use super::super::Player;

impl Player {
    fn trade_request(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let request = match TradeRequestClientPacket::deserialize(&reader) {
                Ok(request) => request,
                Err(e) => {
                    error!("Error deserializing TradeRequestClientPacket {}", e);
                    return;
                }
            };

            map.request_trade(self.id, request.player_id);
        }
    }

    fn trade_accept(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let accept = match TradeAcceptClientPacket::deserialize(&reader) {
                Ok(accept) => accept,
                Err(e) => {
                    error!("Error deserializing TradeAcceptClientPacket {}", e);
                    return;
                }
            };

            map.accept_trade_request(self.id, accept.player_id);
        }
    }

    fn trade_close(&mut self) {
        if let Some(map) = &self.map {
            if let Some(interact_player_id) = self.interact_player_id {
                map.cancel_trade(self.id, interact_player_id);
            }
        }
    }

    fn trade_add(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let add = match TradeAddClientPacket::deserialize(&reader) {
                Ok(add) => add,
                Err(e) => {
                    error!("Error deserializing TradeAddClientPacket {}", e);
                    return;
                }
            };

            map.add_trade_item(self.id, add.add_item);
        }
    }

    fn trade_remove(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let remove = match TradeRemoveClientPacket::deserialize(&reader) {
                Ok(remove) => remove,
                Err(e) => {
                    error!("Error deserializing TradeRemoveClientPacket {}", e);
                    return;
                }
            };

            map.remove_trade_item(self.id, remove.item_id);
        }
    }

    fn trade_agree(&mut self, reader: EoReader) {
        if let Some(map) = &self.map {
            let agree = match TradeAgreeClientPacket::deserialize(&reader) {
                Ok(agree) => agree,
                Err(e) => {
                    error!("Error deserializing TradeAgreeClientPacket {}", e);
                    return;
                }
            };

            if agree.agree {
                map.agree_trade(self.id);
            } else {
                map.disagree_trade(self.id);
            }
        }
    }

    pub fn handle_trade(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.trade_request(reader),
            PacketAction::Accept => self.trade_accept(reader),
            PacketAction::Close => self.trade_close(),
            PacketAction::Add => self.trade_add(reader),
            PacketAction::Remove => self.trade_remove(reader),
            PacketAction::Agree => self.trade_agree(reader),
            _ => error!("Unhandled packet Trade_{:?}", action),
        }
    }
}
