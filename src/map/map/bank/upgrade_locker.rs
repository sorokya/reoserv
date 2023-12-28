use eo::{
    data::{i32, i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn upgrade_locker(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.bank_level >= SETTINGS.bank.max_upgrades {
            return;
        }

        let cost = SETTINGS.bank.upgrade_base_cost
            + SETTINGS.bank.upgrade_cost_step * character.bank_level;

        if character.get_item_amount(1) < cost {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, cost);
        character.bank_level += 1;

        let mut builder = StreamBuilder::new();
        builder.add_int(character.get_item_amount(1));
        builder.add_char(character.bank_level as i32);

        character.player.as_ref().unwrap().send(
            PacketAction::Buy,
            PacketFamily::Locker,
            builder.get(),
        );
    }
}
