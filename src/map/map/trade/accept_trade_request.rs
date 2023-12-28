use eo::{
    data::{i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub async fn accept_trade_request(&mut self, player_id: i32, target_player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let target_player = match target_character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let target_player_interact_player_id = match target_player.get_interact_player_id().await {
            Some(player_id) => player_id,
            None => return,
        };

        if target_player_interact_player_id != player_id {
            return;
        }

        if !in_client_range(&character.coords, &target_character.coords) {
            return;
        }

        player.set_interact_player_id(Some(target_player_id));
        player.set_trading(true);
        target_player.set_trading(true);

        let mut builder = StreamBuilder::new();
        builder.add_short(target_player_id);
        builder.add_break_string(&target_character.name);
        builder.add_short(player_id);
        builder.add_break_string(&character.name);
        player.send(PacketAction::Open, PacketFamily::Trade, builder.get());

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_break_string(&character.name);
        builder.add_short(target_player_id);
        builder.add_break_string(&target_character.name);
        target_player.send(PacketAction::Open, PacketFamily::Trade, builder.get());
    }
}
