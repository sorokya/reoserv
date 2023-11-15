use eo::data::EOShort;

use super::Map;

impl Map {
    pub fn cancel_trade(&mut self, player_id: EOShort, partner_player_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        player.set_trading(false);
        player.set_trade_accepted(false);
        player.set_interact_player_id(None);
        character.trade_items.clear();

        let partner = match self.characters.get_mut(&partner_player_id) {
            Some(partner) => partner,
            None => return,
        };
        partner.trade_items.clear();

        partner.player.as_ref().unwrap().cancel_trade();
    }
}