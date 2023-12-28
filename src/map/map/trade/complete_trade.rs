use eo::{
    data::{i32, StreamBuilder, EO_BREAK_CHAR},
    protocol::{Emote, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub async fn complete_trade(&mut self, player_id: i32, partner_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let trade_items = character.trade_items.clone();

        let partner_character = match self.characters.get(&partner_id) {
            Some(partner_character) => partner_character,
            None => return,
        };

        let partner_trade_items = partner_character.trade_items.clone();

        let mut builder = StreamBuilder::new();

        builder.add_short(player_id);
        let character = self.characters.get_mut(&player_id).unwrap();
        character.trade_items.clear();
        for item in &trade_items {
            builder.add_short(item.id);
            builder.add_int(item.amount);
            character.remove_item(item.id, item.amount);
        }

        let character = self.characters.get_mut(&partner_id).unwrap();
        for item in &trade_items {
            character.add_item(item.id, item.amount);
        }

        builder.add_byte(EO_BREAK_CHAR);

        builder.add_short(partner_id);
        let character = self.characters.get_mut(&partner_id).unwrap();
        character.trade_items.clear();
        for item in &partner_trade_items {
            builder.add_short(item.id);
            builder.add_int(item.amount);
            character.remove_item(item.id, item.amount);
        }

        let character = self.characters.get_mut(&player_id).unwrap();
        for item in &partner_trade_items {
            character.add_item(item.id, item.amount);
        }

        let character = self.characters.get(&player_id).unwrap();
        let partner_character = self.characters.get(&partner_id).unwrap();
        let player = character.player.as_ref().unwrap();
        let partner = partner_character.player.as_ref().unwrap();

        let buf = builder.get();

        player.set_trading(false);
        player.set_trade_accepted(false);
        player.send(PacketAction::Use, PacketFamily::Trade, buf.clone());
        partner.set_trading(false);
        partner.set_trade_accepted(false);
        partner.send(PacketAction::Use, PacketFamily::Trade, buf);

        self.emote(player_id, Emote::Trade);
        self.emote(partner_id, Emote::Trade);
    }
}
