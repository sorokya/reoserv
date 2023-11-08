use std::cmp;

use eo::{
    data::{EOShort, EOThree, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::SETTINGS;

use super::Map;

impl Map {
    pub fn spike_damage(&mut self, player_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let damage = (character.max_hp as f32 * SETTINGS.map.drain_hp_damage).floor() as EOShort;
        let damage = cmp::min(damage, character.hp);

        character.hp -= damage;

        let hp_percentage = character.get_hp_percentage();

        let mut builder = StreamBuilder::new();
        builder.add_char(2);
        builder.add_short(damage);
        builder.add_short(character.hp);
        builder.add_short(character.max_hp);

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.player.as_ref().unwrap().send(
            PacketAction::Spec,
            PacketFamily::Effect,
            builder.get(),
        );

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_char(hp_percentage);
        builder.add_char(if character.hp == 0 { 1 } else { 0 });
        builder.add_three(damage as EOThree);

        self.send_buf_near_player(
            player_id,
            PacketAction::Admin,
            PacketFamily::Effect,
            builder.get(),
        );

        if character.hp == 0 {
            character.player.as_ref().unwrap().die();
        }
    }
}
