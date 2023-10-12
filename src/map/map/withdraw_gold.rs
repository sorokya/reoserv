use std::cmp;

use eo::{
    data::{EOInt, EOShort, EOThree, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub async fn withdraw_gold(&mut self, player_id: EOShort, session_id: EOThree, amount: EOInt) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let actual_session_id = match player.get_session_id().await {
            Ok(session_id) => session_id as EOThree,
            Err(_) => return,
        };

        if session_id != actual_session_id {
            return;
        }

        let amount = cmp::min(character.gold_bank, amount);
        if amount == 0 {
            return;
        }

        let amount = character.can_hold(1, amount);
        if amount == 0 {
            return;
        }

        let interact_npc_index = match player.get_interact_npc_index().await {
            Some(index) => index,
            None => return,
        };

        if !self.npcs.contains_key(&interact_npc_index) {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.gold_bank -= amount;
        character.add_item(1, amount);

        let mut builder = StreamBuilder::new();
        builder.add_int(character.get_item_amount(1));
        builder.add_int(character.gold_bank);

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Bank,
            builder.get(),
        );
    }
}
