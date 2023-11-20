use eo::{
    data::{EOShort, StreamBuilder, EO_BREAK_CHAR},
    protocol::{PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub async fn send_trade_update(&self, player_id: EOShort) {
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

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        for item in character.trade_items.iter() {
            builder.add_short(item.id);
            builder.add_int(item.amount);
        }
        builder.add_byte(EO_BREAK_CHAR);
        builder.add_short(partner_id);
        for item in partner_character.trade_items.iter() {
            builder.add_short(item.id);
            builder.add_int(item.amount);
        }

        let buf = builder.get();

        player.send(PacketAction::Reply, PacketFamily::Trade, buf.clone());
        partner.send(
            if partner_accepted {
                PacketAction::Admin
            } else {
                PacketAction::Reply
            },
            PacketFamily::Trade,
            buf,
        );
    }
}
