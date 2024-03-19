use eolib::protocol::net::{
    server::{TradeAdminServerPacket, TradeItemData, TradeReplyServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub async fn send_trade_update(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let partner_id = match player.get_interact_player_id().await {
            Some(partner_id) => partner_id,
            None => return,
        };

        let partner_character = match self.characters.get(&partner_id) {
            Some(partner_character) => partner_character,
            None => return,
        };

        let partner = match partner_character.player.as_ref() {
            Some(partner) => partner,
            None => return,
        };

        let partner_accepted = partner.is_trade_accepted().await;

        player.send(
            PacketAction::Reply,
            PacketFamily::Trade,
            &TradeReplyServerPacket {
                trade_data: TradeItemData {
                    partner_player_id: partner_id,
                    partner_items: partner_character.trade_items.clone(),
                    your_player_id: player_id,
                    your_items: character.trade_items.clone(),
                },
            },
        );

        if partner_accepted {
            partner.send(
                PacketAction::Admin,
                PacketFamily::Trade,
                &TradeAdminServerPacket {
                    trade_data: TradeItemData {
                        partner_player_id: player_id,
                        partner_items: character.trade_items.clone(),
                        your_player_id: partner_id,
                        your_items: partner_character.trade_items.clone(),
                    },
                },
            );
        } else {
            partner.send(
                PacketAction::Reply,
                PacketFamily::Trade,
                &TradeReplyServerPacket {
                    trade_data: TradeItemData {
                        partner_player_id: player_id,
                        partner_items: character.trade_items.clone(),
                        your_player_id: partner_id,
                        your_items: partner_character.trade_items.clone(),
                    },
                },
            );
        }
    }
}
