use eolib::protocol::net::{
    server::{TradeAgreeServerPacket, TradeSpecServerPacket},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn unaccept_trade(&mut self, player_id: i32, partner_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
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

        player.set_trade_accepted(false);

        player.send(
            PacketAction::Spec,
            PacketFamily::Trade,
            &TradeSpecServerPacket { agree: false },
        );

        partner.send(
            PacketAction::Agree,
            PacketFamily::Trade,
            &TradeAgreeServerPacket {
                partner_player_id: player_id,
                agree: false,
            },
        );
    }
}
