use eolib::protocol::net::{
    server::{TradeAgreeServerPacket, TradeSpecServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub async fn accept_trade(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.trade_items.is_empty() {
            return;
        }

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

        if partner_character.trade_items.is_empty() {
            return;
        }

        let partner = match partner_character.player.as_ref() {
            Some(partner) => partner,
            None => return,
        };

        player.set_trade_accepted(true);

        if partner.is_trade_accepted().await {
            self.complete_trade(player_id, partner_id).await;
            return;
        }

        player.send(
            PacketAction::Spec,
            PacketFamily::Trade,
            &TradeSpecServerPacket { agree: true },
        );

        partner.send(
            PacketAction::Agree,
            PacketFamily::Trade,
            &TradeAgreeServerPacket {
                agree: true,
                partner_player_id: player_id,
            },
        );
    }
}
