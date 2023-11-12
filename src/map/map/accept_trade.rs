use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub async fn accept_trade(&mut self, player_id: EOShort) {
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

        player.set_trade_accepted(true);

        if partner.is_trade_accepted().await {
            self.complete_trade(player_id, partner_id).await;
            return;
        }

        let mut builder = StreamBuilder::new();
        builder.add_char(1);

        player.send(PacketAction::Spec, PacketFamily::Trade, builder.get());

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_char(1);
        partner.send(PacketAction::Agree, PacketFamily::Trade, builder.get());
    }
}
