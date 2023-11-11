use eo::data::EOShort;

use super::Map;

impl Map {
    pub fn cancel_trade(&mut self, player_id: EOShort) {
        for (id, character) in &self.characters {
            if *id == player_id {
                continue;
            }

            character.player.as_ref().unwrap().cancel_trade(player_id);
        }
    }
}
