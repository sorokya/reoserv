use eolib::protocol::net::{
    server::{TradeAgreeServerPacket, TradeSpecServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn accept_trade(&mut self, player_id: i32, partner_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.trade_items.is_empty() {
            return;
        }

        let player = match character.player.as_ref() {
            Some(player) => player.to_owned(),
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
            Some(partner) => partner.to_owned(),
            None => return,
        };

        player.set_trade_accepted(true);

        tokio::spawn(async move {
            if partner.is_trade_accepted().await.expect("Failed to check if trade accepted. Timeout") {
                let map = match partner.get_map().await {
                    Ok(map) => map,
                    Err(e) => {
                        error!("Failed to get map: {}", e);
                        return;
                    }
                };

                map.complete_trade(player_id, partner_id);
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
        });
    }
}
