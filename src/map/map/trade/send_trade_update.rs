use eolib::protocol::net::{
    server::{TradeAdminServerPacket, TradeItemData, TradeReplyServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn send_trade_update(&self, player_id: i32, partner_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player.to_owned(),
            None => return,
        };

        let partner_character = match self.characters.get(&partner_id) {
            Some(partner_character) => partner_character,
            None => return,
        };

        let partner = match partner_character.player.as_ref() {
            Some(partner) => partner.to_owned(),
            None => return,
        };

        let partner_items = partner_character.trade_items.to_owned();
        let your_items = character.trade_items.to_owned();

        tokio::spawn(async move {
            let partner_accepted = partner.is_trade_accepted().await;

            player.send(
                PacketAction::Reply,
                PacketFamily::Trade,
                &TradeReplyServerPacket {
                    trade_data: [
                        TradeItemData {
                            player_id: partner_id,
                            items: partner_items.to_owned(),
                        },
                        TradeItemData {
                            player_id,
                            items: your_items.to_owned(),
                        },
                    ],
                },
            );

            if partner_accepted {
                partner.set_trade_accepted(false);
                partner.send(
                    PacketAction::Admin,
                    PacketFamily::Trade,
                    &TradeAdminServerPacket {
                        trade_data: [
                            TradeItemData {
                                player_id,
                                items: your_items,
                            },
                            TradeItemData {
                                player_id: partner_id,
                                items: partner_items,
                            },
                        ],
                    },
                );
            } else {
                partner.send(
                    PacketAction::Reply,
                    PacketFamily::Trade,
                    &TradeReplyServerPacket {
                        trade_data: [
                            TradeItemData {
                                player_id,
                                items: your_items,
                            },
                            TradeItemData {
                                player_id: partner_id,
                                items: partner_items,
                            },
                        ],
                    },
                );
            }
        });
    }
}
